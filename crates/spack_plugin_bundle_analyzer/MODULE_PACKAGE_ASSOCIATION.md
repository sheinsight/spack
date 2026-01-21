# Module 关联 Package 的设计

## 现状

### 当前数据结构

```rust
// module/module.rs
pub struct Module {
  pub id: String,
  pub name: String,
  pub size: u64,
  pub chunks: Vec<String>,
  pub module_kind: ModuleKind,
  pub module_type: ModuleType,
  pub is_node_module: bool,  // 只有这个字段表示是否来自 node_modules
  pub name_for_condition: String,
  pub concatenated_modules: Option<Vec<ConcatenatedModuleInfo>>,
}

// package/package.rs
pub struct Package {
  pub name: String,
  pub version: String,
  pub size: u64,
  pub module_count: usize,
  pub modules: Vec<String>,  // Package → Module 的关联
  pub package_json_path: String,
}
```

### 当前关联方式

**单向关联**：Package → Module

```rust
// Package 知道它包含哪些 Module
pub struct Package {
  pub modules: Vec<String>,  // Module IDs
}

// Module 不知道它属于哪个 Package
pub struct Module {
  pub is_node_module: bool,  // 只能知道是否来自 node_modules
}
```

---

## 问题：前端如何关联？

### 前端需要写的代码

```typescript
// 1. 预处理：建立索引（性能优化）
const modulePackageMap = new Map<string, Package>();
packages.forEach(pkg => {
  pkg.modules.forEach(moduleId => {
    modulePackageMap.set(moduleId, pkg);  // O(n*m)
  });
});

// 2. 查找模块对应的包
const pkg = modulePackageMap.get(module.id);
if (pkg) {
  console.log(`${module.name} 来自 ${pkg.name}@${pkg.version}`);
}
```

**问题**：
- ❌ 前端需要建立索引（O(n*m) 复杂度）
- ❌ 每个前端项目都要实现相同逻辑
- ❌ 消耗前端内存和 CPU

---

## 方案：Module 直接关联 Package

### 方案 A：存储包名和版本（推荐）

```rust
pub struct Module {
  pub id: String,
  pub name: String,
  pub size: u64,
  pub chunks: Vec<String>,
  pub module_kind: ModuleKind,
  pub module_type: ModuleType,
  pub is_node_module: bool,
  pub name_for_condition: String,
  pub concatenated_modules: Option<Vec<ConcatenatedModuleInfo>>,

  // 🆕 直接存储包信息
  pub package_name: Option<String>,     // 如 "react" 或 "@babel/core"
  pub package_version: Option<String>,  // 如 "18.2.0"
}
```

**优点**：
- ✅ 前端直接访问：`module.package_name`
- ✅ 不需要建立索引
- ✅ 不需要额外查找
- ✅ 数据冗余小（只多了 2 个字段）

**缺点**：
- ⚠️ 数据冗余（同一个包名和版本会在多个 Module 中重复）
- ⚠️ 需要在构建 Module 时关联 Package

---

### 方案 B：存储 Package ID 引用

```rust
pub struct Module {
  // ... 现有字段
  pub package_id: Option<String>,  // 引用 Package.name
}
```

**优点**：
- ✅ 数据冗余最小
- ✅ 类似数据库外键设计

**缺点**：
- ❌ 前端仍需要根据 package_id 查找 Package
- ❌ 相比方案 A 并没有简化太多

---

### 方案 C：替换 `is_node_module` 为 `module_source`

```rust
pub enum ModuleSource {
  Source,      // 源码
  ThirdParty,  // 三方包
  Internal,    // 内部模块
}

pub struct Module {
  // ... 现有字段

  // 替换 is_node_module
  pub module_source: ModuleSource,

  // 如果是 ThirdParty，则有包信息
  pub package_name: Option<String>,
  pub package_version: Option<String>,
}
```

**优点**：
- ✅ 更清晰的分类（三种来源）
- ✅ 替换了不够用的 `is_node_module`
- ✅ 前端可以直接分组

**缺点**：
- ⚠️ 破坏向后兼容（删除了 `is_node_module`）

---

## 对比分析

### 前端代码对比

**现状（需要建立索引）**：
```typescript
// 构建索引：O(n*m)
const map = new Map();
packages.forEach(pkg => {
  pkg.modules.forEach(mid => map.set(mid, pkg));
});

// 使用
modules.forEach(module => {
  const pkg = map.get(module.id);
  if (pkg) {
    console.log(`${module.name} 来自 ${pkg.name}@${pkg.version}`);
  }
});
```

**方案 A（直接访问）**：
```typescript
// 无需建立索引
modules.forEach(module => {
  if (module.package_name) {
    console.log(`${module.name} 来自 ${module.package_name}@${module.package_version}`);
  }
});
```

**性能提升**：
- 建立索引：从 O(n*m) → O(0)
- 查找包信息：从 O(1) 查表 → O(1) 直接访问
- 内存占用：从需要建立 Map → 无需额外内存

---

## 实现方式

### 在哪里关联？

```rust
// modules.rs
impl Modules {
  pub fn from_with_context(
    compilation: &mut Compilation,
    context: &ModuleChunkContext,
  ) -> Self {
    let module_graph = compilation.get_module_graph();

    let modules = module_graph
      .modules()
      .into_iter()
      .map(|(id, module)| {
        let name = module.readable_identifier(&compilation.options.context);
        let is_node_module = name.contains("node_modules/");

        // 🆕 如果是 node_modules，提取包名
        let (package_name, package_version) = if is_node_module {
          extract_package_info(&name)  // 从路径提取
        } else {
          (None, None)
        };

        Module {
          id: id.to_string(),
          name: name.to_string(),
          // ... 其他字段
          is_node_module,
          package_name,      // 🆕
          package_version,   // 🆕
        }
      })
      .collect();

    Modules(modules)
  }
}

/// 从模块路径提取包名和版本（需要实现）
fn extract_package_info(name: &str) -> (Option<String>, Option<String>) {
  // 从路径提取包名
  // ./node_modules/react/index.js -> "react"
  // ./node_modules/@babel/core/lib/index.js -> "@babel/core"

  let parts: Vec<&str> = name.split("node_modules/").collect();
  if parts.len() < 2 {
    return (None, None);
  }

  let after_nm = parts[1];
  let path_parts: Vec<&str> = after_nm.split('/').collect();

  let package_name = if path_parts[0].starts_with('@') && path_parts.len() >= 2 {
    // Scoped package
    Some(format!("{}/{}", path_parts[0], path_parts[1]))
  } else {
    Some(path_parts[0].to_string())
  };

  // 版本号需要从 packages 中查找，或者从路径中解析（pnpm）
  // 暂时返回 None，后续再关联
  (package_name, None)
}
```

### 如何获取版本号？

**方式 1：在构建 Packages 后再回填**
```rust
// 先构建 Modules（没有版本号）
let mut modules = Modules::from_with_context(compilation, context);

// 再构建 Packages
let packages = Packages::from_modules(&modules);

// 回填版本号
for module in &mut modules.0 {
  if let Some(pkg_name) = &module.package_name {
    if let Some(pkg) = packages.iter().find(|p| &p.name == pkg_name) {
      module.package_version = Some(pkg.version.clone());
    }
  }
}
```

**方式 2：传入 Packages 参数**
```rust
impl Modules {
  pub fn from_with_context_and_packages(
    compilation: &mut Compilation,
    context: &ModuleChunkContext,
    packages: &Packages,  // 🆕 传入已构建的 packages
  ) -> Self {
    // 构建时直接关联版本号
  }
}
```

**方式 3：从路径解析版本号（仅 pnpm）**
```rust
// pnpm 路径包含版本号
// ./node_modules/.pnpm/react@18.2.0/node_modules/react/index.js
fn extract_package_version_from_pnpm_path(name: &str) -> Option<String> {
  // 正则匹配 .pnpm/package-name@version/
  // ...
}
```

---

## 数据冗余分析

### 冗余大小估算

假设一个项目：
- 100 个三方包
- 每个包平均 10 个模块
- 总共 1000 个三方模块

**方案 A（存储包名和版本）**：
- 每个 Module 增加：
  - `package_name`: ~20 bytes (如 "@babel/core")
  - `package_version`: ~10 bytes (如 "7.22.0")
  - 总计：30 bytes

- 总冗余：1000 modules × 30 bytes = **30KB**

**对比原始数据大小**：
- 1000 modules × ~500 bytes (Module 结构) = 500KB
- 增加比例：30KB / 500KB = **6%**

**结论**：✅ 数据冗余可接受（只增加 6%）

---

## 对 NAPI 的影响

```rust
// raw_bundle_analyzer.rs
#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsModule {
  pub id: String,
  pub name: String,
  pub size: u32,
  pub chunks: Vec<String>,
  pub module_kind: String,
  pub module_type: String,
  pub is_node_module: bool,
  pub name_for_condition: String,
  pub concatenated_modules: Option<Vec<JsConcatenatedModuleInfo>>,

  // 🆕 添加字段
  pub package_name: Option<String>,
  pub package_version: Option<String>,
}

impl From<Module> for JsModule {
  fn from(value: Module) -> Self {
    Self {
      // ... 现有字段
      package_name: value.package_name,      // 🆕
      package_version: value.package_version, // 🆕
    }
  }
}
```

**影响**：
- ✅ 简单添加两个字段
- ✅ TypeScript 自动生成类型
- ✅ 向后兼容（Optional 字段）

---

## 推荐方案

### 🎯 推荐：方案 A + 方式 1

```rust
pub struct Module {
  // ... 现有字段
  pub is_node_module: bool,           // 保持向后兼容
  pub package_name: Option<String>,    // 🆕
  pub package_version: Option<String>, // 🆕
}
```

**实现步骤**：
1. 在 `Module` 结构添加两个字段
2. 在构建 Module 时从路径提取包名
3. 构建 Packages 后回填版本号
4. 同步到 NAPI bindings

**优点**：
- ✅ 前端直接访问，无需建立索引
- ✅ 向后兼容（保留 `is_node_module`）
- ✅ 数据冗余小（+6%）
- ✅ 实现简单

---

## 迁移路径

### 第 1 步：添加字段（不破坏现有代码）

```rust
pub struct Module {
  pub is_node_module: bool,  // 保留
  pub package_name: Option<String>,    // 新增
  pub package_version: Option<String>, // 新增
}
```

### 第 2 步：前端可以选择使用

```typescript
// 老代码（仍然可用）
if (module.is_node_module) {
  const pkg = packages.find(p => p.modules.includes(module.id));
}

// 新代码（更简单）
if (module.package_name) {
  console.log(`${module.package_name}@${module.package_version}`);
}
```

### 第 3 步：未来可以废弃 `is_node_module`

```rust
// 未来版本
#[deprecated(note = "Use package_name.is_some() instead")]
pub is_node_module: bool,
```

---

## 总结

### 问题回答

**Q: Module 关联上 Package 是否对前端友好很多？**

**A: 是的！非常友好！** ✅

**收益**：
- ✅ 前端无需建立索引（省去 O(n*m) 操作）
- ✅ 代码更简洁（`module.package_name` 直接访问）
- ✅ 性能更好（无需额外内存和 CPU）
- ✅ 逻辑统一（后端处理，前端复用）

**成本**：
- ⚠️ 数据冗余增加 6%（可接受）
- ⚠️ 需要在 Rust 端实现关联逻辑

**结论**：**强烈建议实现！** 🎯

这是一个典型的"用空间换时间"的优化，而且空间成本很低（+6%），时间收益很大（省去前端建立索引）。
