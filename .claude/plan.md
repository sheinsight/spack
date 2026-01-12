# 重复依赖检测插件重构计划

## 问题分析

### 当前实现的问题

1. **数据丢失** (lib.rs:91-102)
   - 第91行按 `(name, version)` 分组,将同版本不同路径的包归为一组
   - 第100行 `libs[0].clone()` 只取每组的第一个,丢失了其他路径的信息
   - **实际影响**: 用户看不到同版本包在多个路径被打包的情况

2. **检测不准确**
   - 同名同版本但在不同路径的包,实际上会被 rspack 打包多次
   - 当前逻辑无法报告这种情况,导致用户误以为没有重复

### 具体案例

**场景**: 项目有以下依赖
```
node_modules/react@18.0.0
legacy/node_modules/react@18.0.0
node_modules/react@17.0.0
```

**当前输出**:
```javascript
{
  name: "react",
  libs: [
    { version: "18.0.0", file: "node_modules/react/package.json" },  // 只有第一个
    { version: "17.0.0", file: "node_modules/react/package.json" }
  ]
}
```

**期望输出**:
```javascript
{
  name: "react",
  libs: [
    { version: "18.0.0", file: "node_modules/react/package.json" },
    { version: "18.0.0", file: "legacy/node_modules/react/package.json" },  // 不应丢失
    { version: "17.0.0", file: "node_modules/react/package.json" }
  ]
}
```

## 解决方案

### 方案设计

**核心思路**: 使用包的完整路径作为唯一标识,而不是 `(name, version)` 组合

### 数据结构变更

**无需修改** - 现有数据结构已经足够:
- `Library` 包含 `file` 字段(完整路径)
- `LibraryGroup` 的 `libs: Vec<Library>` 可以容纳多个实例

### 算法重构

#### 当前逻辑 (lib.rs:89-103)
```rust
cache
  .into_values()
  .into_group_map_by(|lib| (lib.name.clone(), lib.version.clone())) // 按(name,version)分组
  .into_iter()
  .into_group_map_by(|((name, _), _)| name.clone())                 // 按name重新分组
  .into_iter()
  .filter(|(_, libs)| libs.len() > 1)                               // 过滤多版本
  .map(|(name, groups)| LibraryGroup {
    name,
    libs: groups.into_iter().map(|(_, libs)| libs[0].clone()).collect(), // 问题所在
  })
```

#### 新逻辑
```rust
cache
  .into_values()
  .into_group_map_by(|lib| lib.name.clone())  // 只按 name 分组
  .into_iter()
  .filter_map(|(name, libs)| {
    // 情况1: 不同版本 -> 重复依赖
    let unique_versions: Vec<_> = libs.iter().map(|l| &l.version).unique().collect();

    // 情况2: 同版本但多个路径 -> 也是重复(会被打包多次)
    let unique_paths: Vec<_> = libs.iter().map(|l| &l.file).unique().collect();

    // 只有当存在多个版本或多个路径时才算重复
    if unique_versions.len() > 1 || unique_paths.len() > 1 {
      Some(LibraryGroup { name, libs })  // 保留所有实例
    } else {
      None
    }
  })
  .collect()
```

### 实现步骤

#### 步骤 1: 修改核心检测逻辑
**文件**: `crates/spack_plugin_duplicate_dependency/src/lib.rs`
**位置**: 第 89-103 行

**改动**:
```rust
let duplicate_libraries: Vec<LibraryGroup> = cache
  .into_values()
  .into_group_map_by(|lib| lib.name.clone())
  .into_iter()
  .filter_map(|(name, libs)| {
    // 检查是否有多个不同的版本或路径
    let unique_versions: Vec<_> = libs.iter().map(|l| &l.version).unique().collect();
    let unique_paths: Vec<_> = libs.iter().map(|l| &l.file).unique().collect();

    // 有多个版本或多个路径都算重复
    if unique_versions.len() > 1 || unique_paths.len() > 1 {
      Some(LibraryGroup { name, libs })
    } else {
      None
    }
  })
  .collect();
```

**关键点**:
- 移除了 `(name, version)` 的二次分组
- 移除了 `libs[0]` 的数据丢失问题
- 保留所有库实例,包括同版本不同路径的情况

#### 步骤 2: 验证编译
```bash
pnpm run build:dev
```

#### 步骤 3: 编写测试用例
创建测试场景验证:
1. 不同版本的重复检测
2. 同版本不同路径的重复检测
3. 单一版本单一路径不报告

#### 步骤 4: 提交代码
```bash
git add crates/spack_plugin_duplicate_dependency/src/lib.rs
git commit -m "fix: 修复重复依赖检测丢失同版本多路径实例的问题"
```

## 优势分析

### 1. 数据完整性
- ✅ 不会丢失任何路径信息
- ✅ 用户可以看到所有被打包的实例

### 2. 检测准确性
- ✅ 正确识别同版本多路径的重复(实际会被打包多次)
- ✅ 正确识别不同版本的重复

### 3. 性能影响
- ✅ 减少了一次分组操作,性能略有提升
- ✅ 时间复杂度从 O(n log n * 2) 降至 O(n log n)

### 4. 向后兼容
- ✅ 数据结构不变,JavaScript 层无需修改
- ✅ 仅改进了数据的完整性,不影响现有使用方式

## 风险评估

### 低风险
- 数据结构无变化
- 仅修改分组逻辑,不涉及核心编译流程
- 向后兼容

### 需要注意
- 输出的 `libs` 数组可能比之前更长(因为不再丢失数据)
- 用户可能会看到更多"重复"(实际上之前就存在但被隐藏了)

## 替代方案考虑

### 方案 A: 保持当前逻辑,添加路径去重参数
**优点**: 向后兼容性更强
**缺点**: 用户仍可能看不到完整信息

### 方案 B: 当前方案 (推荐)
**优点**:
- 数据完整
- 逻辑简化
- 性能提升

**缺点**:
- 输出数据量可能增加(但这是正确的行为)

## 总结

本次重构将:
1. 修复数据丢失问题
2. 提高检测准确性
3. 简化代码逻辑
4. 保持向后兼容

建议采用方案 B,直接修复当前问题。
