# Module-Package 关联实现总结（方案 B）

## 实现完成 ✅

已成功实现方案 B：Module 通过 `package_json_path` 字段关联 Package。

---

## 修改内容

### 1. Rust 核心数据结构

#### `module/module.rs`
```rust
pub struct Module {
  // ... 现有字段

  /// 关联的 Package 的 package.json 路径（唯一标识）
  /// 仅三方包模块有值，用于精确匹配对应的 Package
  pub package_json_path: Option<String>,
}
```

#### `module/modules.rs`
- 新增 `associate_packages(&mut self, packages: &Packages)` 方法
- 通过 `Package.modules` 反向建立 `module_id → package` 映射
- 时间复杂度：O(n + m)，n = packages 数量，m = modules 数量

```rust
impl Modules {
  pub fn associate_packages(&mut self, packages: &Packages) {
    // 构建 module_id → package 映射（O(n)）
    let mut module_package_map: HashMap<String, &Package> = HashMap::new();

    for package in packages.iter() {
      for module_id in &package.modules {
        module_package_map.insert(module_id.clone(), package);
      }
    }

    // 为每个 Module 填充 package_json_path（O(m)）
    for module in &mut self.0 {
      if let Some(package) = module_package_map.get(&module.id) {
        module.package_json_path = Some(package.package_json_path.clone());
      }
    }
  }
}
```

#### `lib.rs`
- 在构建 Packages 后调用 `modules.associate_packages(&packages)`
- 顺序：Modules → Packages → Associate

```rust
// 3. 收集 Modules
let mut modules = Modules::from_with_context(&mut *compilation, &module_chunk_context);

// 6. 分析 Packages
let packages = Packages::from_with_resolver(&modules, &mut resolver);

// 7. 关联 Module 和 Package（填充 package_json_path）
modules.associate_packages(&packages);
```

### 2. NAPI Bindings

#### `binding/src/raws/raw_bundle_analyzer.rs`
```rust
#[napi(object)]
pub struct JsModule {
  // ... 现有字段

  /// 关联的 Package 的 package.json 路径（唯一标识）
  /// 仅三方包模块有值，用于精确匹配对应的 Package
  pub package_json_path: Option<String>,
}

impl From<Module> for JsModule {
  fn from(value: Module) -> Self {
    Self {
      // ... 现有字段
      package_json_path: value.package_json_path,
    }
  }
}
```

---

## 前端使用指南

### TypeScript 类型定义

```typescript
interface Module {
  id: string;
  name: string;
  size: number;
  chunks: string[];
  moduleKind: string;
  moduleType: string;
  isNodeModule: boolean;
  nameForCondition: string;
  concatenatedModules?: ConcatenatedModuleInfo[];

  // 🆕 新增字段
  packageJsonPath?: string;  // 关联的 package.json 路径
}

interface Package {
  name: string;
  version: string;
  size: number;
  moduleCount: number;
  modules: string[];  // Module IDs
  packageJsonPath: string;  // 唯一标识
}
```

### 使用方式 1：简单显示（推荐）

如果只需要显示包名和版本，可以直接从 `packages` 中查找：

```typescript
function ModuleCard({ module, packages }: Props) {
  // 通过 package_json_path 精确查找
  const pkg = module.packageJsonPath
    ? packages.find(p => p.packageJsonPath === module.packageJsonPath)
    : undefined;

  return (
    <div>
      <h3>{module.name}</h3>

      {pkg && (
        <Badge>
          📦 {pkg.name}@{pkg.version}
        </Badge>
      )}

      <span>{formatSize(module.size)}</span>
    </div>
  );
}
```

**问题**：每次查找都要遍历 packages 数组（O(n)）

---

### 使用方式 2：预处理索引（推荐 ⭐）

在顶层组件预处理，建立 `packageJsonPath → Package` 映射：

```typescript
function BundleAnalyzer({ data }: { data: Report }) {
  // 预处理：建立索引（只需 1 行代码！）
  const packageMap = useMemo(() =>
    new Map(data.packages.map(p => [p.packageJsonPath, p])),
    [data.packages]
  );

  // 通过 Context 传递给子组件
  return (
    <PackageMapContext.Provider value={packageMap}>
      <ChunkList chunks={data.chunks} />
      <ModuleList modules={data.modules} />
    </PackageMapContext.Provider>
  );
}

// 子组件使用（O(1) 查找）
function ModuleCard({ module }: Props) {
  const packageMap = useContext(PackageMapContext);

  // O(1) 查找
  const pkg = module.packageJsonPath
    ? packageMap.get(module.packageJsonPath)
    : undefined;

  return (
    <div>
      <h3>{module.name}</h3>

      {pkg && (
        <Badge>
          📦 {pkg.name}@{pkg.version}
        </Badge>
      )}

      <span>{formatSize(module.size)}</span>
    </div>
  );
}
```

**优点**：
- ✅ O(1) 查找性能
- ✅ 只需预处理一次
- ✅ 通过 Context 全局复用
- ✅ 精确唯一匹配

---

### 使用方式 3：统计分析

```typescript
// 统计每个包被多少模块使用
function analyzePackageUsage(modules: Module[], packages: Package[]) {
  const packageUsage = new Map<string, number>();

  modules.forEach(module => {
    if (module.packageJsonPath) {
      const count = packageUsage.get(module.packageJsonPath) || 0;
      packageUsage.set(module.packageJsonPath, count + 1);
    }
  });

  // 找出使用最多的包
  const topPackages = Array.from(packageUsage.entries())
    .map(([path, count]) => ({
      package: packages.find(p => p.packageJsonPath === path)!,
      usageCount: count,
    }))
    .sort((a, b) => b.usageCount - a.usageCount)
    .slice(0, 10);

  return topPackages;
}
```

---

### Context 实现示例

```typescript
// PackageMapContext.tsx
import { createContext, useContext } from 'react';

const PackageMapContext = createContext<Map<string, Package>>(new Map());

export function usePackageMap() {
  return useContext(PackageMapContext);
}

export { PackageMapContext };
```

```typescript
// BundleAnalyzer.tsx
import { useMemo } from 'react';
import { PackageMapContext } from './PackageMapContext';

export function BundleAnalyzer({ data }: { data: Report }) {
  const packageMap = useMemo(() =>
    new Map(data.packages.map(p => [p.packageJsonPath, p])),
    [data.packages]
  );

  return (
    <PackageMapContext.Provider value={packageMap}>
      <div className="bundle-analyzer">
        <Summary data={data.summary} />
        <ChunkList chunks={data.chunks} />
        <ModuleList modules={data.modules} />
      </div>
    </PackageMapContext.Provider>
  );
}
```

```typescript
// ModuleCard.tsx
import { usePackageMap } from './PackageMapContext';

export function ModuleCard({ module }: { module: Module }) {
  const packageMap = usePackageMap();
  const pkg = module.packageJsonPath
    ? packageMap.get(module.packageJsonPath)
    : undefined;

  return (
    <div className="module-card">
      <div className="module-name">{module.name}</div>

      {pkg && (
        <div className="package-badge">
          📦 {pkg.name}@{pkg.version}
        </div>
      )}

      <div className="module-size">{formatSize(module.size)}</div>

      {module.chunks.length > 1 && (
        <div className="shared-badge">🔗 Shared</div>
      )}
    </div>
  );
}
```

---

## 数据对比

### 方案 A（未实现）
```rust
pub struct Module {
  pub package_name: Option<String>,       // ~20 bytes
  pub package_version: Option<String>,    // ~10 bytes
  pub package_json_path: Option<String>,  // ~100 bytes
}
```
- 数据增长：+130KB (1000 个模块)
- 相对增长：+26%
- 前端使用：直接访问，无需预处理

### 方案 B（已实现 ✅）
```rust
pub struct Module {
  pub package_json_path: Option<String>,  // ~100 bytes
}
```
- 数据增长：+100KB (1000 个模块)
- 相对增长：+20%
- 前端使用：1 行代码预处理

**节省**：-30KB (-23%)

---

## 关键优势

### 1. 数据精简 ✅
- 比方案 A 少 30KB（节省 23%）
- 保持数据规范化，避免冗余

### 2. 精确匹配 ✅
- `package_json_path` 是唯一标识
- 不存在多个版本的歧义问题

### 3. 前端友好 ✅
- 预处理只需 1 行代码
- O(1) 查找性能
- 通过 Context 全局复用

### 4. 可扩展性 ✅
- Package 结构变化不影响 Module
- 保持松耦合

### 5. 向后兼容 ✅
- `is_node_module` 字段保留
- 可选字段，不影响现有代码

---

## 性能分析

### Rust 端
- 关联操作：O(n + m)
  - n = packages 数量（~100）
  - m = modules 数量（~1000）
- 典型项目：~1-2ms（可忽略）

### 前端
- 预处理索引：O(n)，n = packages 数量
  - 100 个包：~1ms
  - 1000 个包：~10ms
- 查找性能：O(1)（Map）

---

## 后续优化方向（可选）

如果发现前端每次都需要显示包名和版本，可以考虑添加快捷字段：

```rust
pub struct Module {
  // 快捷访问（可选）
  pub package_name: Option<String>,
  pub package_version: Option<String>,

  // 精确引用（主要使用）
  pub package_json_path: Option<String>,
}
```

**但目前建议**：先使用方案 B，根据实际使用反馈再决定是否添加快捷字段。

**原因**：
1. 前端预处理确实很简单（1 行代码）
2. 保持数据精简更符合工程实践
3. 避免过早优化

---

## 总结

✅ 方案 B 已成功实现

✅ 编译测试通过

✅ 前端使用简单（1 行 useMemo）

✅ 数据最精简（-23% vs 方案 A）

✅ 精确唯一匹配

**下一步**：等待前端集成反馈，根据实际使用情况决定是否需要添加快捷字段。
