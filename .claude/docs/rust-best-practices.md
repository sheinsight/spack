# Rust 最佳实践指南

本文档基于 spack 项目代码库的审查,总结了 Rust 开发的最佳实践和改进建议。

## 目录
- [代码组织与模块结构](#代码组织与模块结构)
- [错误处理](#错误处理)
- [性能优化](#性能优化)
- [类型设计与 API](#类型设计与-api)
- [代码质量与惯用法](#代码质量与惯用法)
- [具体改进建议](#具体改进建议)

---

## 代码组织与模块结构

### ✅ 优秀实践

#### 1. 模块化代码组织
最近完成的重构很好地演示了如何将大型文件拆分成子模块:

```rust
// 之前: asset.rs (120+ 行)
pub struct Asset { ... }
pub struct Assets(Vec<Asset>);
impl From<&mut Compilation> for Assets { ... }

// 之后: asset/mod.rs
mod asset;
mod assets;
pub use asset::Asset;
pub use assets::Assets;

// asset/asset.rs - 结构定义
pub struct Asset { ... }

// asset/assets.rs - 集合和逻辑
pub struct Assets(Vec<Asset>);
impl From<&mut Compilation> for Assets { ... }
```

**优点**:
- 职责分离明确
- 每个文件专注单一功能
- 易于维护和测试
- 更好的代码导航

#### 2. 使用 `mod.rs` 作为模块入口

```rust
// src/asset/mod.rs
mod asset;
mod assets;

pub use asset::Asset;
pub use assets::Assets;
```

**最佳实践**: `mod.rs` 只做重新导出,实现代码放在单独文件中。

#### 3. 清晰的公共 API 设计

```rust
// lib.rs - 明确的公共 API
pub use crate::{
  asset::Asset,
  chunk::Chunk,
  module::Module,
  // ... 其他公共类型
};
```

### ⚠️ 需要改进

#### 1. 避免循环依赖风险

**当前代码**:
```rust
// module/modules.rs
use crate::context::ModuleChunkContext;

// context.rs
use rspack_core::Compilation;
```

**建议**: 定期使用 `cargo tree` 检查依赖关系,确保没有循环依赖。

#### 2. 考虑功能分层

建议的目录结构:
```
src/
├── lib.rs                   # 公共 API
├── types/                   # 核心类型定义
│   ├── asset.rs
│   ├── chunk.rs
│   └── module.rs
├── collectors/              # 数据收集器
│   ├── assets.rs
│   ├── chunks.rs
│   └── modules.rs
├── analyzers/               # 分析器
│   ├── chunk_overlap.rs
│   └── duplicate_packages.rs
└── utils/                   # 工具函数
    ├── context.rs
    └── resolver.rs
```

---

## 错误处理

### ✅ 优秀实践

#### 1. 使用 Result 类型传播错误

```rust
// lib.rs:159
fs::write(f, format!("{:#?}", report)).await?;

// lib.rs:166-168
if let Some(on_analyzed) = &self.options.on_analyzed {
  if let Err(e) = on_analyzed(report).await {
    tracing::error!("BundleAnalyzerPlugin callback failed: {:?}", e);
  }
}
```

**优点**:
- 使用 `?` 简化错误传播
- 对回调错误进行日志记录而不中断流程

### ⚠️ 需要改进

#### 1. 静默忽略错误

**问题代码** (asset/assets.rs:91-93):
```rust
if encoder.write_all(data).is_err() {
  return None;  // ❌ 静默失败
}
```

**改进建议**:
```rust
if let Err(e) = encoder.write_all(data) {
  tracing::warn!("Failed to write gzip data: {}", e);
  return None;
}

// 或者更好:
encoder.write_all(data).ok()?;  // 使用 ? 操作符
```

#### 2. 使用 unwrap 存在风险

**问题代码** (lib.rs:158):
```rust
let dir = current_dir().unwrap();  // ❌ 可能 panic
```

**改进建议**:
```rust
let dir = current_dir().map_err(|e| {
  rspack_error::Error::from(format!("Failed to get current directory: {}", e))
})?;
```

#### 3. 不一致的错误处理

**问题代码** (case_sensitive_paths/lib.rs:147-149):
```rust
let Ok(source) = std::fs::read_to_string(issuer) else {
  return Ok(None);  // 静默失败
};
```

**改进建议**:
```rust
let source = std::fs::read_to_string(issuer).map_err(|e| {
  tracing::debug!("Failed to read source file {}: {}", issuer, e);
  e
}).ok()?;
```

### 📚 错误处理最佳实践

1. **永远不要静默忽略错误** - 至少记录日志
2. **避免 unwrap/expect** - 除非在明确安全的情况下
3. **使用自定义错误类型** - 为不同错误场景提供更好的上下文
4. **传播错误而非吞掉** - 让调用者决定如何处理

**推荐错误处理模式**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnalyzerError {
  #[error("Failed to compress file {filename}: {source}")]
  CompressionError {
    filename: String,
    source: std::io::Error,
  },
  #[error("Invalid package.json at {path}: {reason}")]
  InvalidPackageJson {
    path: String,
    reason: String,
  },
}
```

---

## 性能优化

### ✅ 优秀实践

#### 1. 预构建映射避免重复遍历 ⭐⭐⭐⭐⭐

**优秀设计** (context.rs:16-53):
```rust
pub struct ModuleChunkContext {
  pub module_to_chunks: HashMap<Identifier, Vec<String>>,
  pub chunk_to_modules: HashMap<String, (Vec<String>, u64)>,
}

impl From<&Compilation> for ModuleChunkContext {
  fn from(compilation: &Compilation) -> Self {
    // 只遍历一次 chunk_graph，同时构建双向映射
    // ...
  }
}
```

**优点**:
- 从 O(N×M) 优化到 O(N+M)
- 一次遍历构建双向映射
- 后续查询 O(1)

**性能提升**: 显著,特别是大型项目中

#### 2. 使用 Rayon 并行处理 ⭐⭐⭐⭐⭐

**优秀实现** (asset/assets.rs:28-52):
```rust
let assets = assets
  .par_iter()  // 并行迭代器
  .map(|(name, size, buffer_opt)| {
    let gzip_size = if let Some(buffer) = buffer_opt {
      calculate_gzip_size(buffer)  // CPU 密集型任务
    } else {
      None
    };
    // ...
  })
  .collect();
```

**优点**:
- 自动利用多核 CPU
- 对 CPU 密集型任务(如压缩)效果显著
- 几乎零成本并行化

#### 3. 缓存避免重复计算 ⭐⭐⭐⭐

**优秀实现** (package_version_resolver.rs:19-64):
```rust
pub struct PackageVersionResolver {
  cache: HashMap<String, PackageInfo>,
}

pub fn resolve(&mut self, module_path: &str) -> Option<PackageInfo> {
  // 查缓存
  if let Some(info) = self.cache.get(&cache_key) {
    return Some(info.clone());
  }
  // 计算并缓存
  let info = self.find_package_info(dir)?;
  self.cache.insert(cache_key, info.clone());
  Some(info)
}
```

**优点**:
- 避免重复文件 I/O
- 特别适合有大量重复查询的场景

#### 4. 快速压缩级别用于估算 ⭐⭐⭐⭐

**聪明优化** (asset/assets.rs:85-88):
```rust
// 使用压缩级别 1(最快),而非默认的级别 6
// 对于大小估算来说,速度更重要,且大小差异在可接受范围内
let mut encoder = GzEncoder::new(Vec::new(), Compression::new(1));
```

**优点**:
- 速度提升 3-5 倍
- 大小估算误差 < 5%
- 合理的性能/精度权衡

### ⚠️ 可以改进

#### 1. 避免不必要的克隆

**问题代码** (duplicate_packages/duplicate_packages.rs:39-49):
```rust
let chunks = asset_to_chunks
  .get(name.as_str())
  .cloned()  // ❌ 每次都克隆 Vec<String>
  .unwrap_or_default();
```

**改进建议 1 - 使用引用**:
```rust
let chunks = asset_to_chunks
  .get(name.as_str())
  .map(|v| v.as_slice())
  .unwrap_or(&[]);
```

**改进建议 2 - 使用 Cow**:
```rust
use std::borrow::Cow;

let chunks: Cow<[String]> = asset_to_chunks
  .get(name.as_str())
  .map(|v| Cow::Borrowed(v.as_slice()))
  .unwrap_or(Cow::Owned(vec![]));
```

#### 2. 字符串拼接优化

**问题代码**:
```rust
for chunk in chunks {
  let chunk_id = chunk.id().map(|id| id.to_string()).unwrap_or_default();
  // 多次使用 chunk_id.clone()
}
```

**改进建议**:
```rust
// 如果只需要引用,使用 &str
let chunk_id = chunk.id()
  .map(|id| id.as_ref())
  .unwrap_or("");
```

#### 3. 预分配容量

**问题代码** (duplicate_packages/duplicate_packages.rs:22):
```rust
let mut grouped: HashMap<String, Vec<&Package>> = HashMap::new();
```

**改进建议**:
```rust
let mut grouped: HashMap<String, Vec<&Package>> =
  HashMap::with_capacity(packages.len() / 10);  // 估算容量
```

#### 4. 使用 FxHashMap 替代标准 HashMap

**当前**:
```rust
use std::collections::HashMap;
```

**改进建议**:
```rust
use rustc_hash::FxHashMap;  // 性能提升 20-50%

// String key 适合用 FxHashMap
let mut map: FxHashMap<String, Value> = FxHashMap::default();
```

**注意**: FxHashMap 不是加密安全的,只用于内部数据结构。

### 📊 性能优化检查清单

- [ ] 是否有重复遍历大集合?
- [ ] CPU 密集型任务是否可以并行化?
- [ ] 是否有重复的文件 I/O 或计算?
- [ ] 是否有不必要的克隆?
- [ ] HashMap 是否需要预分配容量?
- [ ] 是否可以用 FxHashMap 替代标准 HashMap?
- [ ] 字符串操作是否可以用引用替代所有权?

---

## 类型设计与 API

### ✅ 优秀实践

#### 1. 使用 newtype 模式

```rust
#[derive(Debug, Default, Deref, Into)]
pub struct Assets(Vec<Asset>);

#[derive(Debug, Default, Deref, Into)]
pub struct Chunks(Vec<Chunk>);
```

**优点**:
- 类型安全
- 可以为集合添加特定方法
- 清晰的语义

#### 2. Builder 模式

```rust
// 虽然代码中没有显示完整实现,但 PackageBuilder 的使用很好
package_map
  .entry(info.path.clone())
  .or_insert_with(|| PackageBuilder::new(info))
  .add_module(module);
```

#### 3. 配置对象与合理默认值

```rust
#[derive(Debug, Clone)]
pub struct ChunkOverlapConfig {
  pub min_module_size: u64,
  pub min_duplication_count: usize,
  // ...
}

impl Default for ChunkOverlapConfig {
  fn default() -> Self {
    Self {
      min_module_size: 1024,  // 合理的默认值
      min_duplication_count: 2,
      // ...
    }
  }
}
```

### ⚠️ 可以改进

#### 1. 考虑使用 Cow 减少克隆

**当前**:
```rust
pub struct OverlappedModule {
  pub module_id: String,
  pub module_name: String,
  pub chunks: Vec<String>,
  // ...
}
```

**改进建议**:
```rust
use std::borrow::Cow;

pub struct OverlappedModule<'a> {
  pub module_id: Cow<'a, str>,
  pub module_name: Cow<'a, str>,
  pub chunks: Cow<'a, [String]>,
  // ...
}
```

#### 2. 使用 NonZeroU64 表示非零值

**当前**:
```rust
pub struct ChunkOverlapConfig {
  pub min_module_size: u64,  // 应该 > 0
  pub min_duplication_count: usize,  // 应该 >= 2
}
```

**改进建议**:
```rust
use std::num::NonZeroU64;

pub struct ChunkOverlapConfig {
  pub min_module_size: NonZeroU64,  // 编译时保证 > 0
  pub min_duplication_count: usize,
}
```

#### 3. 考虑使用枚举而非 bool

**问题模式**:
```rust
pub struct Chunk {
  pub entry: bool,
  pub initial: bool,
  pub async_chunks: bool,
  pub runtime: bool,
}
```

**改进建议**:
```rust
pub enum ChunkKind {
  Entry,
  Initial,
  Async,
  Runtime,
  Regular,
}

pub struct Chunk {
  pub kind: ChunkKind,
  // 或者如果可能多个属性
  pub kinds: HashSet<ChunkKind>,
}
```

#### 4. API 一致性

**当前**: 有些方法接受 `&[T]`,有些接受 `&Vec<T>`

**改进建议**: 统一使用切片 `&[T]`,更灵活

```rust
// 好的 API 设计
impl ChunkOverlapAnalysis {
  pub fn from(chunks: &[Chunk], modules: &[Module]) -> Self {
    // ...
  }
}

// 而不是
impl ChunkOverlapAnalysis {
  pub fn from(chunks: &Vec<Chunk>, modules: &Vec<Module>) -> Self {
    // ...
  }
}
```

---

## 代码质量与惯用法

### ✅ 优秀实践

#### 1. 丰富的文档注释

```rust
/// 计算 gzip 压缩后的大小
///
/// 参数:
/// - data: 原始数据字节
///
/// 返回: 压缩后的字节数,如果压缩失败返回 None
///
/// 注意: 使用快速压缩级别(1)以提升性能,因为我们只需要大小估算值
fn calculate_gzip_size(data: &[u8]) -> Option<usize> {
  // ...
}
```

#### 2. 使用 Iterator 组合子

```rust
// duplicate_packages.rs:89-105
let duplicate_libraries: Vec<LibraryGroup> = cache
  .into_values()
  .into_group_map_by(|lib| lib.name.clone())
  .into_iter()
  .filter_map(|(name, libs)| {
    // ...
  })
  .collect();
```

#### 3. 模式匹配与早期返回

```rust
// package_version_resolver.rs:42-44
if !module_path.contains("node_modules/") {
  return None;
}
```

### ⚠️ 可以改进

#### 1. 避免嵌套过深

**问题代码** (case_sensitive_paths/lib.rs:168-178):
```rust
if let Some(dependencies) = package_json.dependencies {
  for item in dependencies.keys() {
    dep_key_set.insert(item.to_string());
  }
}

if let Some(dev_dependencies) = package_json.dev_dependencies {
  for item in dev_dependencies.keys() {
    dep_key_set.insert(item.to_string());
  }
}
```

**改进建议**:
```rust
let add_deps = |deps: Option<_>, set: &mut HashSet<_>| {
  if let Some(deps) = deps {
    set.extend(deps.keys().map(ToString::to_string));
  }
};

add_deps(package_json.dependencies, &mut dep_key_set);
add_deps(package_json.dev_dependencies, &mut dep_key_set);
```

#### 2. 使用 if let 链

**当前代码** (duplicate_dependency/lib.rs:70-82):
```rust
let library = paths.iter().find_map(|p| {
  if let Ok(package_json) = PackageJsonParser::parse(p)
    && let Some(name) = package_json.name
    && let Some(version) = package_json.version
    && let Some(path) = package_json.__raw_path
  {
    return Some(Library::new(/*...*/));
  }
  None
});
```

**已经很好!** 这是 Rust 1.64+ 的 if let 链,非常清晰。

#### 3. 避免不必要的 to_string()

**问题代码**:
```rust
let chunk_id = chunk.id().map(|id| id.to_string()).unwrap_or_default();
// 如果只是临时使用,不需要分配
```

**改进建议**:
```rust
// 如果可能,使用引用
if let Some(id) = chunk.id() {
  // 直接使用 id
}
```

#### 4. 使用 matches! 宏

**可以改进的模式**:
```rust
if module_type == ModuleType::JavaScript || module_type == ModuleType::TypeScript {
  // ...
}
```

**改进建议**:
```rust
if matches!(module_type, ModuleType::JavaScript | ModuleType::TypeScript) {
  // ...
}
```

---

## 具体改进建议

### 高优先级 (建议立即修复)

#### 1. 移除 unwrap (lib.rs:158)

**当前**:
```rust
let dir = current_dir().unwrap();
```

**修复**:
```rust
let dir = current_dir().map_err(|e| {
  rspack_error::Error::from(format!("Failed to get current directory: {}", e))
})?;
```

#### 2. 改进错误日志 (asset/assets.rs:91-93, 99)

**当前**:
```rust
if encoder.write_all(data).is_err() {
  return None;
}
// ...
Err(e) => {
  tracing::error!("{}", e);  // 缺少上下文
  None
}
```

**修复**:
```rust
if let Err(e) = encoder.write_all(data) {
  tracing::warn!("Failed to write data for gzip compression: {}", e);
  return None;
}
// ...
Err(e) => {
  tracing::error!("Failed to finish gzip compression: {}", e);
  None
}
```

#### 3. 添加边界检查 (duplicate_packages/duplicate_packages.rs:59)

**当前**:
```rust
let largest_size = versions[0].size;  // 可能 panic
```

**修复**:
```rust
let largest_size = versions.first()
  .map(|v| v.size)
  .unwrap_or(0);
```

### 中优先级 (建议在下次重构时修复)

#### 1. 使用 FxHashMap 提升性能

```rust
// 替换所有 HashMap<String, _>
use rustc_hash::FxHashMap;

// 之前
let mut map: HashMap<String, Vec<String>> = HashMap::new();

// 之后
let mut map: FxHashMap<String, Vec<String>> = FxHashMap::default();
```

#### 2. 减少克隆

查找并优化所有 `.cloned()` 调用,考虑:
- 是否可以用引用?
- 是否可以用 `Cow`?
- 是否可以移动所有权?

#### 3. 添加单元测试

当前插件缺少单元测试,建议添加:

```rust
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_calculate_gzip_size() {
    let data = b"Hello, World!";
    let size = calculate_gzip_size(data);
    assert!(size.is_some());
    assert!(size.unwrap() < data.len());
  }

  #[test]
  fn test_detect_duplicates() {
    // ...
  }
}
```

### 低优先级 (可选优化)

#### 1. 考虑异步 I/O

当前使用同步文件 I/O,可以考虑:
```rust
use tokio::fs;

// 并发读取多个文件
let futures: Vec<_> = files
  .iter()
  .map(|f| fs::read_to_string(f))
  .collect();
let results = futures::future::join_all(futures).await;
```

#### 2. 添加基准测试

```rust
// benches/performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_gzip_compression(c: &mut Criterion) {
  let data = vec![0u8; 1024 * 100]; // 100KB
  c.bench_function("gzip_100kb", |b| {
    b.iter(|| calculate_gzip_size(black_box(&data)))
  });
}

criterion_group!(benches, bench_gzip_compression);
criterion_main!(benches);
```

---

## 总结

### 🎯 核心要点

1. **错误处理**: 永远不要静默忽略错误,避免 unwrap
2. **性能优化**: 预构建映射、并行处理、缓存
3. **类型设计**: 使用 newtype、枚举、配置对象
4. **代码质量**: 文档注释、单元测试、惯用法

### 📈 优先级路线图

**第一阶段**: 修复高优先级问题
- [ ] 移除所有 unwrap
- [ ] 改进错误日志
- [ ] 添加边界检查

**第二阶段**: 性能优化
- [ ] 使用 FxHashMap
- [ ] 减少不必要的克隆
- [ ] 预分配容量

**第三阶段**: 提升代码质量
- [ ] 添加单元测试
- [ ] 添加基准测试
- [ ] 改进文档

### 🔗 相关资源

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)
