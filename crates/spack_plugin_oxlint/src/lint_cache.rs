use std::sync::{
  Arc,
  atomic::{AtomicBool, AtomicUsize, Ordering},
};

use dashmap::{DashMap, DashSet};
use oxc::diagnostics;
use oxc_linter::Message;

/// Lint 缓存管理器（优化版）
///
/// 负责管理三个层次的缓存和状态：
/// 1. **初始化标志** (`initialized`): 标识插件是否是首次运行（应用级别，整个生命周期只初始化一次）
/// 2. **本轮已检查文件集** (`linted_files`): 跟踪当前编译周期内已经 lint 过的文件（周期级别，每次 compilation 开始时清空）
/// 3. **Lint 结果缓存** (`cache`): 存储所有文件的 lint 错误信息（持久级别，跨编译周期保持）
/// 4. **错误计数器** (`error_count`): 原子计数器，实时维护总错误数（O(1) 查询）
///
/// ## 性能优化
///
/// ### 使用 DashMap 替代 Mutex<HashMap>
/// - **无锁并发读**: 多个线程可以同时读取，无需等待锁
/// - **细粒度锁**: DashMap 内部使用分片锁（shard），不同分片可以并发写入
/// - **更高吞吐**: 在高并发场景下性能显著优于 Mutex<HashMap>
///
/// ### 使用 AtomicUsize 维护错误计数
/// - **O(1) 查询**: `get_error_count()` 从 O(n) 遍历降至 O(1) 原子读
/// - **无锁操作**: 读取计数无需加锁，零开销
/// - **实时更新**: 在 `insert_cache`/`remove_from_cache` 时同步更新计数
///
/// ### 使用 DashSet 替代 DashMap<String, ()>
/// - **语义清晰**: `DashSet<String>` 明确表达集合语义，不需要 `()` 占位值
/// - **API 简洁**: `insert(key)` 直接返回 bool，相比 `insert(key, ()).is_none()` 更直观
/// - **无锁并发**: 与 DashMap 相同的性能优势
///
/// ### 使用 AtomicBool 替代 Mutex<bool>
/// - **原子 CAS 操作**: `mark_as_initialized_once()` 使用 compare_exchange 原子性地检查并修改
/// - **无锁**: 不需要 Mutex，更轻量
///
/// ## 缓存调度逻辑
///
/// ### 首次启动流程（mark_as_initialized_once() = true）
/// ```text
/// 1. this_compilation hook 触发
///    ├─> mark_as_initialized_once() 返回 true（首次启动）
///    ├─> clear_linted_files() 清空 linted_files
///    ├─> 遍历项目所有文件
///    │   ├─> mark_files_as_linted() 批量标记文件为已检查
///    │   ├─> 逐个调用 lint_runner.lint()
///    │   └─> insert_cache() 存储 lint 结果（自动更新计数器）
///    └─> 不在此处读取错误计数（等待 finish_modules）
///
/// 2. succeed_module hook 触发（每个模块编译成功后）
///    ├─> try_mark_as_linted() 检查并标记文件
///    └─> 返回 false（在步骤1已标记）→ 跳过，避免重复 lint
///
/// 3. finish_modules hook 触发（所有模块处理完成后）
///    └─> get_error_count() O(1) 读取最终计数器
/// ```
///
/// ### 热更新流程（mark_as_initialized_once() = false）
/// ```text
/// 1. this_compilation hook 触发（文件变更后）
///    ├─> mark_as_initialized_once() 返回 false（非首次）
///    ├─> clear_linted_files() 清空 linted_files（开启新周期）
///    └─> 不执行全量 lint（跳过遍历文件步骤）
///
/// 2. succeed_module hook 触发（变更的模块重新编译）
///    ├─> try_mark_as_linted() 原子性地检查并标记
///    │   └─> 返回 true（首次标记）→ 执行 lint
///    ├─> 执行 lint_runner.lint() 检查该文件
///    └─> 更新 cache（自动更新计数器）
///        ├─> 有错误: insert_cache() → 计数器 +new -old
///        └─> 无错误: remove_from_cache() → 计数器 -old
///
/// 3. finish_modules hook 触发（所有模块处理完成后）
///    └─> get_error_count() O(1) 读取最终计数器（准确反映本轮结果）
/// ```
///
/// ## 并发安全与性能
///
/// - **DashMap/DashSet**: 无锁并发读，细粒度写锁，适合高并发场景
/// - **AtomicBool/AtomicUsize**: CPU 级别的原子操作，无需系统锁
/// - **原子 CAS**: `try_mark_as_linted()` 使用 DashSet 的原子 insert 防止竞态条件
#[derive(Debug)]
pub struct LintCache {
  /// 初始化标志（应用级别）- 使用 AtomicBool 替代 Mutex<bool>
  ///
  /// - `false`: 插件首次运行，需要执行全量 lint
  /// - `true`: 后续运行，只需增量 lint
  ///
  /// **优化**: 使用原子 bool，`mark_as_initialized_once()` 通过 compare_exchange 实现无锁操作
  initialized: Arc<AtomicBool>,

  /// 当前编译周期已检查的文件集（周期级别）- 使用 DashSet 替代 Mutex<HashSet>
  ///
  /// - 每次 `this_compilation` 开始时通过 `clear_linted_files()` 清空
  /// - 用于防止同一编译周期内重复 lint 同一文件
  /// - `DashSet` 是 `DashMap<K, ()>` 的语义化封装，API 更简洁
  ///
  /// **优化**: DashSet 支持无锁并发读，细粒度写锁
  linted_files: Arc<DashSet<String>>,

  /// Lint 结果缓存（持久级别）- 使用 DashMap 替代 Mutex<HashMap>
  ///
  /// - 键: 文件路径
  /// - 值: 该文件的所有 lint 错误/警告消息
  /// - 跨编译周期保持，用于快速统计总错误数
  ///
  /// **优化**: DashMap 提供无锁并发访问，性能显著优于 Mutex<HashMap>
  cache: Arc<DashMap<String, Vec<Message>>>,

  /// 总错误数计数器（实时维护）
  ///
  /// - 在 `insert_cache`/`remove_from_cache` 时同步更新
  /// - `get_error_count()` 直接读取，O(1) 时间复杂度
  ///
  /// **优化**: 原子计数器，无需遍历所有文件，避免锁竞争
  error_count: Arc<AtomicUsize>,
}

impl LintCache {
  /// 创建新的缓存实例
  pub fn new() -> Self {
    Self {
      initialized: Arc::new(AtomicBool::new(false)),
      linted_files: Arc::new(DashSet::new()),
      cache: Arc::new(DashMap::new()),
      error_count: Arc::new(AtomicUsize::new(0)),
    }
  }

  /// 检查并标记为已初始化（原子 CAS 操作，无锁）
  ///
  /// **行为**:
  /// - 第一次调用: 返回 `true`，并将 `initialized` 标记为 `true`
  /// - 后续调用: 返回 `false`（已经初始化过）
  ///
  /// **语义**: 此方法不是纯查询方法，而是带副作用的状态转换操作
  /// - 名字体现了"标记"的副作用
  /// - 返回值表示是否成功完成了首次标记
  ///
  /// **实现细节**:
  /// - 使用 `compare_exchange` 原子 CAS (Compare-And-Swap) 操作
  /// - 如果当前值是 `false`，则设置为 `true` 并返回 `Ok(false)`（旧值）
  /// - 如果当前值是 `true`，则返回 `Err(true)`（当前值）
  ///
  /// **性能**:
  /// - 无需 Mutex 锁，CPU 级别的原子操作
  /// - 避免了锁竞争和上下文切换
  ///
  /// **示例**:
  /// ```rust,no_run
  /// # use std::sync::Arc;
  /// # use std::sync::atomic::{AtomicBool, Ordering};
  /// # struct LintCache { initialized: Arc<AtomicBool> }
  /// # impl LintCache {
  /// #     fn mark_as_initialized_once(&self) -> bool {
  /// #         self.initialized.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok()
  /// #     }
  /// # }
  /// # let cache = LintCache { initialized: Arc::new(AtomicBool::new(false)) };
  /// // 首次调用（完成初始化标记）
  /// let is_first = cache.mark_as_initialized_once(); // true
  ///
  /// // 再次调用（已经初始化过）
  /// let is_first = cache.mark_as_initialized_once(); // false
  /// ```
  pub fn mark_as_initialized_once(&self) -> bool {
    // compare_exchange: 原子性地比较并交换
    // 参数: (期望值, 新值, 成功时的内存顺序, 失败时的内存顺序)
    self
      .initialized
      .compare_exchange(
        false,            // 期望当前值是 false
        true,             // 如果是，则设置为 true
        Ordering::SeqCst, // 成功时使用顺序一致性
        Ordering::SeqCst, // 失败时使用顺序一致性
      )
      .is_ok() // Ok 表示成功从 false 改为 true（首次标记）
  }

  /// 清空当前编译周期的已检查文件集
  ///
  /// **时机**: 在每次 `this_compilation` hook 开始时调用
  ///
  /// **作用**:
  /// - 开启新的编译周期
  /// - 允许 `succeed_module` 重新 lint 文件
  ///
  /// **性能**: DashMap 的 clear() 是分片清空，比 Mutex<HashSet> 更高效
  pub fn clear_linted_files(&self) {
    self.linted_files.clear();
  }

  /// 原子性地尝试标记文件为已检查
  ///
  /// **时机**: 在 `succeed_module` 中决定是否 lint 某个文件时调用
  ///
  /// **返回**:
  /// - `true`: 首次标记，需要执行 lint
  /// - `false`: 已存在，跳过 lint（防止重复）
  ///
  /// **优势**:
  /// - 原子操作：`DashSet::insert` 返回 bool，直接表示是否首次插入
  /// - 避免竞态：不需要先 `contains` 再 `insert`，一次操作完成
  /// - 语义清晰：相比 `DashMap<K, ()>`，DashSet 的 API 更简洁直观
  ///
  /// **示例**:
  /// ```rust,no_run
  /// # use std::sync::Arc;
  /// # use dashmap::DashSet;
  /// # struct LintCache { linted_files: Arc<DashSet<String>> }
  /// # impl LintCache {
  /// #     fn try_mark_as_linted(&self, path: String) -> bool {
  /// #         self.linted_files.insert(path)
  /// #     }
  /// # }
  /// # let cache = LintCache { linted_files: Arc::new(DashSet::new()) };
  /// # let resource = "file.rs";
  /// // 在 succeed_module 中
  /// if cache.try_mark_as_linted(resource.to_string()) {
  ///     // 首次标记，需要执行 lint
  ///     // let messages = lint_runner.lint(resource).await?;
  ///     // cache.insert_cache(resource.to_string(), messages);
  ///     println!("File needs linting: {}", resource);
  /// }
  /// // 如果返回 false，则跳过（已经 lint 过了）
  /// ```
  pub fn try_mark_as_linted(&self, path: String) -> bool {
    // DashSet::insert 返回 bool
    // true 表示首次插入（需要 lint）
    // false 表示已存在（跳过 lint）
    self.linted_files.insert(path)
  }

  /// 批量标记多个文件为已检查
  ///
  /// **时机**: 在首次启动的 `this_compilation` 中全量 lint 前调用
  ///
  /// **作用**: 避免后续 `succeed_module` 重复 lint 这些文件
  ///
  /// **性能**: DashSet 支持并发插入，多个线程可以同时标记不同文件
  pub fn mark_files_as_linted(&self, files: &[String]) {
    for file in files {
      self.linted_files.insert(file.clone());
    }
  }

  /// 将文件的 lint 结果存入缓存，并更新错误计数器
  ///
  /// **时机**: lint 完成后，发现有错误/警告时调用
  ///
  /// **作用**:
  /// - 持久化 lint 结果
  /// - 同步更新 `error_count` 计数器（原子操作）
  ///
  /// **实现细节**:
  /// 1. 计算新消息中的错误数量
  /// 2. 插入新消息，获取旧消息（如果有）
  /// 3. 如果有旧消息，计算旧错误数量
  /// 4. 更新计数器：`+新错误数 -旧错误数`
  ///
  /// **性能**:
  /// - DashMap 插入: 细粒度锁，只锁定单个分片
  /// - 原子计数: fetch_add/fetch_sub 无需额外锁
  pub fn insert_cache(&self, path: String, messages: Vec<Message>) {
    // 计算新消息中的错误数量
    let new_error_count = messages
      .iter()
      .filter(|m| m.error.severity == diagnostics::Severity::Error)
      .count();

    // 插入新消息，获取旧消息
    if let Some(old_messages) = self.cache.insert(path, messages) {
      // 有旧消息，计算旧错误数量
      let old_error_count = old_messages
        .iter()
        .filter(|m| m.error.severity == diagnostics::Severity::Error)
        .count();

      // 更新计数器：先加新的，再减旧的（避免中间状态为负数）
      self
        .error_count
        .fetch_add(new_error_count, Ordering::Relaxed);
      self
        .error_count
        .fetch_sub(old_error_count, Ordering::Relaxed);
    } else {
      // 没有旧消息，直接加上新错误数
      self
        .error_count
        .fetch_add(new_error_count, Ordering::Relaxed);
    }
  }

  /// 从缓存中移除文件的 lint 结果，并更新错误计数器
  ///
  /// **时机**: 热更新中 lint 某文件后，发现无错误时调用
  ///
  /// **作用**: 清除该文件之前的旧错误（文件已修复），同步更新计数器
  ///
  /// **性能**: DashMap 的 remove 返回被移除的值，一次操作完成
  pub fn remove_from_cache(&self, path: &str) {
    // DashMap 的 remove 返回 Option<(K, V)>
    if let Some((_, old_messages)) = self.cache.remove(path) {
      // 计算被移除消息的错误数量
      let old_error_count = old_messages
        .iter()
        .filter(|m| m.error.severity == diagnostics::Severity::Error)
        .count();

      // 从计数器中减去
      self
        .error_count
        .fetch_sub(old_error_count, Ordering::Relaxed);
    }
  }

  /// 获取总错误数（O(1) 原子读取，无锁）
  ///
  /// **时机**: 在 `finish_modules` hook 中调用，生成 rspack diagnostic
  ///
  /// **返回**: 错误数量（只统计 `Severity::Error`，不包括警告）
  ///
  /// **时序保证**:
  /// - 在 `finish_modules` 中调用可确保所有 `succeed_module` 已完成
  /// - 返回值准确反映当前编译周期的最终错误状态
  ///
  /// **性能对比**:
  /// - **旧实现**: O(n) 遍历所有文件的所有消息 + Mutex 锁
  /// - **新实现**: O(1) 原子读取 + 零锁开销
  ///
  /// **实现细节**:
  /// - 计数器在 `insert_cache`/`remove_from_cache` 时实时维护
  /// - 读取时只需一次原子 load 操作，无需遍历
  ///
  /// **示例**:
  /// ```rust,no_run
  /// # use std::sync::Arc;
  /// # use std::sync::atomic::{AtomicUsize, Ordering};
  /// # struct LintCache { error_count: Arc<AtomicUsize> }
  /// # impl LintCache {
  /// #     fn get_error_count(&self) -> usize {
  /// #         self.error_count.load(Ordering::Relaxed)
  /// #     }
  /// # }
  /// # let cache = LintCache { error_count: Arc::new(AtomicUsize::new(0)) };
  /// // 旧实现 (O(n) + 锁):
  /// // cache.lock().map(|c| c.values().flatten().filter(...).count())
  ///
  /// // 新实现 (O(1) + 无锁):
  /// let count = cache.get_error_count();
  /// assert_eq!(count, 0);
  /// ```
  pub fn get_error_count(&self) -> usize {
    // 原子读取，使用 Relaxed 顺序（最低开销）
    // Relaxed: 只保证原子性，不保证与其他操作的顺序
    // 对于计数器场景足够（最终一致性）
    self.error_count.load(Ordering::Relaxed)
  }
}
