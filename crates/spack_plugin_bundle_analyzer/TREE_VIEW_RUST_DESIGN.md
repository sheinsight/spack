# Chunk 树视图 - Rust Struct 设计

## 核心需求

用户交互：
1. 点击 Chunk 展开查看它包含的所有 Modules
2. 如果 Module 是 Concatenated 类型，可以继续展开查看内部模块
3. 需要区分：**源码** 还是 **三方包**

---

## 基础 Struct（已有）

```rust
/// 代码块（已存在）
#[derive(Debug)]
pub struct Chunk {
  pub id: String,
  pub names: Vec<String>,
  pub size: u64,
  pub modules: Vec<String>,  // 模块 ID 列表
  pub entry: bool,
  pub initial: bool,
  pub async_chunks: bool,
  pub runtime: bool,
  pub reason: String,
  pub files: Vec<String>,
  pub parents: Vec<String>,
  pub children: Vec<String>,
}

/// 模块（已存在）
#[derive(Debug)]
pub struct Module {
  pub id: String,
  pub name: String,
  pub size: u64,
  pub chunks: Vec<String>,  // 所属 chunk IDs
  pub module_kind: ModuleKind,
  pub module_type: ModuleType,
  pub is_node_module: bool,
  pub name_for_condition: String,
  pub concatenated_modules: Option<Vec<ConcatenatedModuleInfo>>,
}

/// 合并模块信息（已存在）
#[derive(Debug, Clone)]
pub struct ConcatenatedModuleInfo {
  pub id: String,
  pub name: String,
  pub size: u64,
}

/// npm 包（已存在）
#[derive(Debug)]
pub struct Package {
  pub name: String,
  pub version: String,
  pub size: u64,
  pub module_count: usize,
  pub modules: Vec<String>,  // 模块 ID 列表
  pub package_json_path: String,
}
```

---

## 树视图专用 Struct 设计

### 方案 1：最简设计（直接复用现有数据）

```rust
/// Chunk 树视图
#[derive(Debug, Clone)]
#[napi(object)]
pub struct ChunkTreeView {
  /// Chunk 基础信息
  pub chunk: Chunk,
  /// 该 chunk 包含的所有模块
  pub modules: Vec<Module>,
}
```

**优点**：
- ✅ 简单，直接复用现有数据结构
- ✅ 前端可以自己分组和过滤

**缺点**：
- ❌ `Module.is_node_module` 只是 bool，无法区分 "源码" vs "三方包" vs "内部模块"
- ❌ 三方包模块无法直接知道包名和版本
- ❌ 无法直接知道模块是否被共享
- ❌ 前端需要自己解析路径，容易出错

---

### 方案 2：增强设计（添加额外信息）

```rust
/// Chunk 树视图
#[derive(Debug, Clone)]
#[napi(object)]
pub struct ChunkTreeView {
  /// Chunk 基础信息
  pub chunk: Chunk,
  /// 模块树节点列表
  pub module_nodes: Vec<ModuleTreeNode>,
}

/// 模块来源分类
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleSource {
  /// 源码（用户代码）
  Source,
  /// 三方包（node_modules）
  ThirdParty,
  /// 内部模块（webpack runtime 等）
  Internal,
}

/// 三方包信息
#[derive(Debug, Clone)]
#[napi(object)]
pub struct PackageInfo {
  /// 包名（如 "react" 或 "@babel/core"）
  pub name: String,
  /// 版本号
  pub version: String,
}

/// 模块树节点
#[derive(Debug, Clone)]
#[napi(object)]
pub struct ModuleTreeNode {
  /// 模块基础信息
  pub module: Module,

  /// 模块来源分类
  pub source: String,  // "Source" | "ThirdParty" | "Internal"

  /// 包信息（仅三方包有值）
  pub package: Option<PackageInfo>,

  /// 是否被多个 chunk 共享
  pub is_shared: bool,

  /// 子节点（仅 Concatenated Module 有）
  pub children: Option<Vec<ModuleTreeNode>>,
}
```

**优点**：
- ✅ 清晰的三种来源分类
- ✅ 三方包直接关联包名和版本
- ✅ 一眼看出模块是否被共享
- ✅ 后端统一处理，前端无需解析路径

**缺点**：
- ❌ 数据结构稍复杂
- ❌ 需要额外的构建逻辑

---

### 方案 3：分层设计（更细粒度）

```rust
/// Chunk 树视图
#[derive(Debug, Clone)]
#[napi(object)]
pub struct ChunkTreeView {
  /// Chunk 基础信息
  pub chunk: Chunk,
  /// 源码模块
  pub source_modules: Vec<ModuleTreeNode>,
  /// 三方包（按包分组）
  pub third_party_packages: Vec<PackageGroup>,
  /// 内部模块
  pub internal_modules: Vec<ModuleTreeNode>,
}

/// 包分组
#[derive(Debug, Clone)]
#[napi(object)]
pub struct PackageGroup {
  /// 包名
  pub name: String,
  /// 版本号
  pub version: String,
  /// 该包的所有模块
  pub modules: Vec<ModuleTreeNode>,
}

/// 模块树节点
#[derive(Debug, Clone)]
#[napi(object)]
pub struct ModuleTreeNode {
  pub module: Module,
  pub is_shared: bool,
  pub children: Option<Vec<ModuleTreeNode>>,
}
```

**优点**：
- ✅ 后端直接分好组
- ✅ 前端直接渲染，无需额外处理
- ✅ 数据结构直接对应 UI 结构

**缺点**：
- ❌ 结构最复杂
- ❌ 灵活性较低（前端想改分组方式需要后端改）

---

## 关键设计问题

### 问题 1：如何判断模块来源？

```rust
impl ModuleSource {
  pub fn from_module_name(name: &str) -> Self {
    if name.contains("node_modules/") {
      ModuleSource::ThirdParty
    } else if name.starts_with("webpack/runtime/") ||
              name.starts_with("(webpack)") ||
              name.starts_with("webpack/bootstrap") {
      ModuleSource::Internal
    } else {
      ModuleSource::Source
    }
  }
}
```

**问题**：
- `Module.name` 的格式是什么？
- 是否所有三方包都包含 `node_modules/`？
- webpack runtime 的路径格式是否固定？

### 问题 2：如何提取包信息？

```rust
fn extract_package_name(module_name: &str) -> Option<String> {
  // 例如：./node_modules/react/index.js -> "react"
  // 例如：./node_modules/@babel/core/lib/index.js -> "@babel/core"

  let parts: Vec<&str> = module_name.split("node_modules/").collect();
  if parts.len() < 2 {
    return None;
  }

  let after_nm = parts[1];
  let path_parts: Vec<&str> = after_nm.split('/').collect();

  // Scoped package
  if path_parts[0].starts_with('@') && path_parts.len() >= 2 {
    Some(format!("{}/{}", path_parts[0], path_parts[1]))
  } else {
    Some(path_parts[0].to_string())
  }
}
```

**问题**：
- 这个逻辑是否覆盖所有情况？
- 是否有其他特殊的包名格式？

### 问题 3：如何关联 Module 和 Package？

**方式 1：通过路径解析**
```rust
let package_name = extract_package_name(&module.name);
let package = packages.iter().find(|p| p.name == package_name);
```

**方式 2：通过 Package.modules**
```rust
let package = packages.iter().find(|p| p.modules.contains(&module.id));
```

**问题**：
- 哪种方式更可靠？
- Package.modules 是否总是包含所有模块？

### 问题 4：Concatenated Module 的内部模块如何处理？

```rust
// ConcatenatedModuleInfo 只有基础信息
pub struct ConcatenatedModuleInfo {
  pub id: String,
  pub name: String,
  pub size: u64,
}

// 是否需要也判断内部模块的来源？
// 是否需要递归构建 ModuleTreeNode？
```

**问题**：
- 内部模块是否也需要 `source` 分类？
- 内部模块是否可能也是 Concatenated？（应该不会）

---

## 推荐方案

我倾向于 **方案 2（增强设计）**，原因：

### 1. 平衡了复杂度和灵活性
- 不像方案 1 那样把所有逻辑推给前端
- 不像方案 3 那样过度限制前端的展示方式

### 2. 提供核心信息
```rust
pub struct ModuleTreeNode {
  pub module: Module,        // 完整的模块信息
  pub source: String,        // 来源分类
  pub package: Option<PackageInfo>,  // 包信息（三方包）
  pub is_shared: bool,       // 是否共享
  pub children: Option<Vec<ModuleTreeNode>>,  // Concatenated 子节点
}
```

这些信息：
- **source**: 让前端可以分组显示
- **package**: 让前端可以显示包名和版本
- **is_shared**: 让前端可以标记共享模块
- **children**: 支持 Concatenated Module 展开

### 3. 前端仍有灵活性
- 可以选择扁平列表 + 标签
- 可以选择分组展示
- 可以自定义排序和过滤

---

## 需要讨论的点

### 1. 字段命名
```rust
// 当前命名
pub source: String,
pub package: Option<PackageInfo>,

// 备选命名
pub module_source: String,
pub package_info: Option<PackageInfo>,
```

### 2. 是否需要统计信息
```rust
pub struct ChunkTreeView {
  pub chunk: Chunk,
  pub module_nodes: Vec<ModuleTreeNode>,

  // 是否需要这些？
  pub total_source_modules: usize,
  pub total_third_party_modules: usize,
  pub total_internal_modules: usize,

  pub source_size: u64,
  pub third_party_size: u64,
  pub internal_size: u64,
}
```

### 3. ModuleSource 是否暴露给 JS
```rust
// 方案 A：使用字符串
pub source: String,  // "Source" | "ThirdParty" | "Internal"

// 方案 B：暴露枚举
#[napi]
pub enum ModuleSource {
  Source,
  ThirdParty,
  Internal,
}

pub source: ModuleSource,
```

### 4. 内部模块的处理
```rust
// Concatenated 的子节点是否需要完整的 ModuleTreeNode？
pub children: Option<Vec<ModuleTreeNode>>,

// 还是简化版本？
pub children: Option<Vec<InnerModule>>,

pub struct InnerModule {
  pub id: String,
  pub name: String,
  pub size: u64,
  pub source: String,
}
```

---

## 你的意见？

请告诉我：
1. 你倾向于哪个方案（1/2/3）？
2. 字段设计是否合理？缺少什么信息？
3. 对于上述 4 个讨论点，你的偏好是什么？
4. 还有其他需要考虑的场景吗？
