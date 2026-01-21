# 前端能否用现有数据构建树视图？

## 现有数据结构

```rust
pub struct Report {
  pub assets: Vec<Asset>,
  pub modules: Vec<Module>,
  pub chunks: Vec<Chunk>,
  pub packages: Vec<Package>,
}

pub struct Chunk {
  pub id: String,
  pub modules: Vec<String>,  // 模块 ID 列表
  // ...
}

pub struct Module {
  pub id: String,
  pub name: String,
  pub size: u64,
  pub chunks: Vec<String>,
  pub module_kind: ModuleKind,
  pub module_type: ModuleType,
  pub is_node_module: bool,
  pub concatenated_modules: Option<Vec<ConcatenatedModuleInfo>>,
  // ...
}

pub struct Package {
  pub name: String,
  pub version: String,
  pub modules: Vec<String>,  // 模块 ID 列表
  // ...
}
```

---

## 前端构建树视图的步骤

### 步骤 1：获取 Chunk 的所有 Modules ✅

```typescript
function getChunkModules(chunkId: string, data: Report): Module[] {
  const chunk = data.chunks.find(c => c.id === chunkId);
  if (!chunk) return [];

  // 通过 chunk.modules 的 ID 列表找到对应的 Module 对象
  return data.modules.filter(m =>
    chunk.modules.includes(m.id)
  );
}
```

**结论**：✅ **可行**，有 `Chunk.modules` 字段

---

### 步骤 2：判断模块来源 ⚠️

```typescript
enum ModuleSource {
  Source = 'Source',
  ThirdParty = 'ThirdParty',
  Internal = 'Internal',
}

function getModuleSource(module: Module): ModuleSource {
  // 方式 1：使用 is_node_module 字段
  if (module.is_node_module) {
    return ModuleSource.ThirdParty;
  }

  // 方式 2：解析 module.name
  if (module.name.includes('node_modules/')) {
    return ModuleSource.ThirdParty;
  }

  if (module.name.startsWith('webpack/runtime/') ||
      module.name.startsWith('(webpack)')) {
    return ModuleSource.Internal;
  }

  return ModuleSource.Source;
}
```

**问题**：
- `Module.is_node_module` 只是 bool，不能区分 "源码" vs "内部模块"
- 需要前端解析 `module.name` 字符串
- webpack runtime 的路径格式是否稳定？可能因版本而异

**结论**：⚠️ **部分可行**，但依赖字符串解析，不够可靠

---

### 步骤 3：找到对应的 Package ✅

```typescript
function findModulePackage(
  module: Module,
  packages: Package[]
): Package | undefined {
  // 方式 1：通过 Package.modules 查找
  return packages.find(pkg =>
    pkg.modules.includes(module.id)
  );
}
```

**性能问题**：
- 每个模块都要遍历所有 packages：O(n * m)
- 如果有 1000 个模块和 100 个包，就是 100,000 次比较

**优化**：可以预先构建索引
```typescript
// 预处理：构建 moduleId -> package 映射
const modulePackageMap = new Map<string, Package>();
packages.forEach(pkg => {
  pkg.modules.forEach(moduleId => {
    modulePackageMap.set(moduleId, pkg);
  });
});

// 查找：O(1)
const pkg = modulePackageMap.get(module.id);
```

**结论**：✅ **可行**，但需要优化

---

### 步骤 4：提取包名和版本 ⚠️

```typescript
function extractPackageInfo(module: Module, packages: Package[]) {
  // 方式 1：通过 Package.modules 关联（推荐）
  const pkg = packages.find(p => p.modules.includes(module.id));
  if (pkg) {
    return {
      name: pkg.name,
      version: pkg.version,
    };
  }

  // 方式 2：解析 module.name（fallback）
  // ./node_modules/react/index.js -> "react"
  // ./node_modules/@babel/core/lib/index.js -> "@babel/core"
  const match = module.name.match(/node_modules\/(@[^/]+\/[^/]+|[^/]+)/);
  if (match) {
    const pkgName = match[1];
    const pkg = packages.find(p => p.name === pkgName);
    return pkg ? {
      name: pkg.name,
      version: pkg.version,
    } : {
      name: pkgName,
      version: 'unknown',
    };
  }

  return null;
}
```

**问题**：
- 正则表达式可能遗漏边界情况
- Scoped package 处理复杂
- 如果 Package.modules 不完整，可能找不到

**结论**：⚠️ **可行但容易出错**

---

### 步骤 5：判断是否被共享 ✅

```typescript
function isModuleShared(module: Module): boolean {
  return module.chunks.length > 1;
}
```

**结论**：✅ **完全可行**，有 `Module.chunks` 字段

---

### 步骤 6：展开 Concatenated Module ✅

```typescript
function buildModuleTree(module: Module): ModuleTreeNode {
  const children = module.concatenated_modules?.map(inner => ({
    id: inner.id,
    name: inner.name,
    size: inner.size,
    source: getModuleSource({ name: inner.name } as Module),
  }));

  return {
    module,
    source: getModuleSource(module),
    package: extractPackageInfo(module, packages),
    isShared: isModuleShared(module),
    children,
  };
}
```

**结论**：✅ **可行**，有 `Module.concatenated_modules` 字段

---

## 完整的前端实现

```typescript
interface ChunkTreeView {
  chunk: Chunk;
  nodes: ModuleTreeNode[];
}

interface ModuleTreeNode {
  module: Module;
  source: ModuleSource;
  package?: {
    name: string;
    version: string;
  };
  isShared: boolean;
  children?: ModuleTreeNode[];
}

function buildChunkTree(
  chunkId: string,
  data: Report
): ChunkTreeView {
  const chunk = data.chunks.find(c => c.id === chunkId);
  if (!chunk) {
    throw new Error(`Chunk ${chunkId} not found`);
  }

  // 1. 预处理：构建 moduleId -> package 映射（性能优化）
  const modulePackageMap = new Map<string, Package>();
  data.packages.forEach(pkg => {
    pkg.modules.forEach(moduleId => {
      modulePackageMap.set(moduleId, pkg);
    });
  });

  // 2. 获取该 chunk 的所有模块
  const chunkModules = data.modules.filter(m =>
    chunk.modules.includes(m.id)
  );

  // 3. 构建树节点
  const nodes = chunkModules.map(module => {
    return buildModuleTreeNode(module, modulePackageMap);
  });

  return { chunk, nodes };
}

function buildModuleTreeNode(
  module: Module,
  packageMap: Map<string, Package>
): ModuleTreeNode {
  // 判断来源
  const source = getModuleSource(module);

  // 获取包信息
  const pkg = packageMap.get(module.id);
  const packageInfo = pkg ? {
    name: pkg.name,
    version: pkg.version,
  } : undefined;

  // 判断是否共享
  const isShared = module.chunks.length > 1;

  // 处理 Concatenated 子节点
  const children = module.concatenated_modules?.map(inner => ({
    module: {
      id: inner.id,
      name: inner.name,
      size: inner.size,
      // ... 其他字段用默认值
    } as Module,
    source: getModuleSourceByName(inner.name),
    package: undefined,  // 简化：内部模块不关联包
    isShared: false,
    children: undefined,
  }));

  return {
    module,
    source,
    package: packageInfo,
    isShared,
    children,
  };
}

function getModuleSource(module: Module): ModuleSource {
  // 优先使用 is_node_module
  if (module.is_node_module) {
    return ModuleSource.ThirdParty;
  }

  return getModuleSourceByName(module.name);
}

function getModuleSourceByName(name: string): ModuleSource {
  if (name.includes('node_modules/')) {
    return ModuleSource.ThirdParty;
  }

  if (name.startsWith('webpack/runtime/') ||
      name.startsWith('(webpack)') ||
      name.includes('webpack/bootstrap')) {
    return ModuleSource.Internal;
  }

  return ModuleSource.Source;
}
```

---

## 总结对比

| 功能 | 是否可行 | 依赖 | 问题 |
|------|---------|------|------|
| 获取 Chunk 的 Modules | ✅ | `Chunk.modules` | 无 |
| 判断模块来源 | ⚠️ | `Module.is_node_module` + 解析 name | 需要解析字符串，不可靠 |
| 找到对应的 Package | ✅ | `Package.modules` | 需要优化性能 |
| 提取包名和版本 | ✅ | `Package.modules` | 如果关联失败需要解析路径 |
| 判断是否共享 | ✅ | `Module.chunks` | 无 |
| 展开 Concatenated | ✅ | `Module.concatenated_modules` | 无 |

---

## 结论

### ✅ 前端**可以**用现有数据构建树视图

但存在以下**问题**：

### 1. 字符串解析不可靠

```typescript
// 这些逻辑容易出错
if (name.includes('node_modules/')) { ... }
if (name.startsWith('webpack/runtime/')) { ... }

// 问题：
// - webpack runtime 路径格式可能变化
// - 第三方包路径格式可能多样
// - Scoped package 处理复杂
```

### 2. 性能开销

```typescript
// 每次查找包都要遍历
packages.find(p => p.modules.includes(module.id))

// 需要预处理建立索引
const modulePackageMap = new Map();
packages.forEach(pkg => {
  pkg.modules.forEach(moduleId => {
    modulePackageMap.set(moduleId, pkg);
  });
});
```

### 3. 逻辑重复

- 每个前端项目都要实现相同的逻辑
- 容易出现不一致的实现
- 维护成本高

### 4. 数据一致性风险

```typescript
// 不同开发者可能写出不同的判断逻辑
// 开发者 A：
if (name.includes('node_modules/')) { ... }

// 开发者 B：
if (name.match(/\/node_modules\//)) { ... }

// 开发者 C：
if (module.is_node_module) { ... }
```

---

## 对比：如果在 Rust 端提供

### Rust 端处理

```rust
pub struct ModuleTreeNode {
  pub module: Module,
  pub source: String,        // 统一的判断逻辑
  pub package: Option<PackageInfo>,  // 统一的关联逻辑
  pub is_shared: bool,       // 预先计算
  pub children: Option<Vec<ModuleTreeNode>>,
}
```

**优势**：
- ✅ 判断逻辑统一，Rust 端维护
- ✅ 性能更好（一次处理，多次使用）
- ✅ 类型安全
- ✅ 前端代码更简洁

**劣势**：
- ❌ 增加 Rust 代码复杂度
- ❌ 前端灵活性稍降低

---

## 我的建议

### 如果时间紧张 → 前端自己处理

**适用场景**：
- 快速验证 UI 效果
- 原型开发
- 一次性项目

**代码示例**：
```typescript
// 前端构建树视图
const tree = buildChunkTree(chunkId, reportData);
```

### 如果追求质量 → Rust 端提供

**适用场景**：
- 长期维护的项目
- 需要高性能
- 多个前端项目复用同一个 API

**代码示例**：
```typescript
// 直接使用 Rust 端构建好的树
const tree = await api.getChunkTree(chunkId);
```

---

## 实际建议的分阶段方案

### 阶段 1：先让前端实现（验证需求）

```typescript
// 快速实现，验证 UI 效果和用户体验
function buildChunkTree(chunkId: string, data: Report) {
  // 前端自己处理
}
```

**目标**：
- 确认 UI 设计是否合理
- 确认需要哪些信息
- 发现遗漏的需求

### 阶段 2：再优化到 Rust 端（性能优化）

```rust
// 确认需求后，将稳定的逻辑移到 Rust
pub struct ChunkTreeView {
  pub chunk: Chunk,
  pub module_nodes: Vec<ModuleTreeNode>,
}
```

**目标**：
- 统一逻辑
- 提升性能
- 减少前端复杂度

---

## 最终回答

**是的，前端可以用现有数据拼凑出树视图。**

但需要：
1. 解析 `module.name` 判断来源
2. 遍历 `packages` 关联包信息
3. 建立索引优化性能

**权衡**：
- **现在前端处理** → 快速验证，但逻辑分散
- **Rust 端提供** → 统一逻辑，但增加后端工作量

**我的建议**：
- 如果只是快速验证 → 前端自己处理
- 如果是长期项目 → Rust 端提供树视图结构

你倾向哪种？
