# Chunk 结构字段扩展分析

本文档分析 Chunk 数据结构的潜在字段扩展建议。

## 优先级说明

- **高优先级 (High)**: 显著提升分析能力，实现成本合理
- **中优先级 (Medium)**: 有价值但实现成本较高，或使用场景相对受限
- **低优先级 (Low)**: 价值较小或实现成本过高

---

## 当前 Chunk 结构

```rust
pub struct Chunk {
  pub id: String,
  pub names: Vec<String>,
  pub size: u64,
  pub modules: Vec<String>,
  pub entry: bool,
  pub initial: bool,
  pub reason: String,
  pub files: Vec<String>,
  pub async_chunks: bool,
  pub runtime: bool,
}
```

---

## 1. parents: Vec<String> 🔶 中优先级

### 实现复杂度

- **复杂度**: 低到中等
- **实现方式**: 从 rspack 的 `ChunkGroup` 中提取父 chunk 关系
- **代码量**: 约 40-70 行
- **关键 API**: `chunk_group.parents` 或通过 `chunk_graph` 遍历
- **ID 来源**: 使用 `ukey.as_u32().to_string()`，与当前 chunks.id 保持一致

```rust
// 实现示例
pub fn collect_parent_chunks(
  chunk_ukey: &ChunkUkey,
  compilation: &Compilation
) -> Vec<String> {
  let chunk_group_by_ukey = &compilation.chunk_group_by_ukey;
  let chunk = compilation.chunk_by_ukey.get(chunk_ukey).unwrap();

  let mut parents = Vec::new();

  // 遍历 chunk 所属的 chunk groups
  for group_ukey in chunk.groups() {
    if let Some(group) = chunk_group_by_ukey.get(group_ukey) {
      // 获取父 chunk groups
      for parent_group_ukey in group.parents() {
        if let Some(parent_group) = chunk_group_by_ukey.get(parent_group_ukey) {
          // 获取父 group 中的 chunks
          for parent_chunk_ukey in parent_group.chunks() {
            // 使用 ukey 作为 ID，确保与 chunks 列表一致
            parents.push(parent_chunk_ukey.as_u32().to_string());
          }
        }
      }
    }
  }

  // 去重
  parents.sort();
  parents.dedup();
  parents
}
```

### 重要说明

**ID 存储方式**：
- ✅ 使用 `ukey.as_u32().to_string()` 获取 chunk ID
- ❌ 不使用 `chunk.id()`（可能为 None 导致空字符串）
- 这确保 `parents` 中的 ID 能在外层 `chunks` 列表中正确匹配

**存储内容**：
- 存储**父 chunk 的 ID 列表**
- 表示哪些 chunk 引用/依赖了当前 chunk
- 例如：入口 chunk `main` 异步加载了 `lazy-route`，则 `lazy-route.parents` 包含 `main` 的 ID

### 增加的数据量

- **每个 Chunk**: 取决于 chunk 层级结构
  - 入口 chunk: 0 个 parents，0 字节
  - 异步 chunk: 1-3 个 parents，约 20-60 字节
  - 深层异步 chunk: 可能更多
- **典型项目** (20 个 chunks，平均 1 个 parent): 约 400 字节
- **大型项目** (200 个 chunks): 约 4KB
- **增长率**: 对总数据量影响 < 0.5%

### 性能开销

- **采集阶段**: 低
  - rspack 内部已维护 chunk 依赖关系
  - 仅需读取现有数据结构
  - 时间复杂度: O(n)，n = chunk 数量
  - 典型开销: < 5ms
- **内存开销**: 低
- **传输开销**: 可忽略

### 可实现功能列表

1. **Chunk 依赖图**:
   - 可视化 chunk 加载顺序
   - 展示 chunk 层级关系
2. **加载顺序优化**:
   - 识别关键 chunk 路径
   - 优化 preload/prefetch 策略
3. **Chunk 拆分分析**:
   - 评估当前 chunk 拆分策略
   - 识别不合理的 chunk 依赖
4. **性能瓶颈识别**:
   - 找出加载链过长的 chunk
   - 建议合并某些 parent-child chunk
5. **并行加载优化**:
   - 识别可以并行加载的 chunk
   - 优化 chunk 依赖结构
6. **代码拆分策略**:
   - 基于依赖关系调整拆分粒度
   - 避免过度拆分导致的请求数激增

---

## 2. children: Vec<String> 🔶 中优先级

### 实现复杂度

- **复杂度**: 低到中等
- **实现方式**: 与 parents 同时从 rspack 提取
- **代码量**: 约 40-70 行（通常与 parents 一起实现）
- **ID 来源**: 使用 `ukey.as_u32().to_string()`，与当前 chunks.id 保持一致

```rust
// 实现示例
pub fn collect_child_chunks(
  chunk_ukey: &ChunkUkey,
  compilation: &Compilation
) -> Vec<String> {
  let chunk_group_by_ukey = &compilation.chunk_group_by_ukey;
  let chunk = compilation.chunk_by_ukey.get(chunk_ukey).unwrap();

  let mut children = Vec::new();

  // 遍历 chunk 所属的 chunk groups
  for group_ukey in chunk.groups() {
    if let Some(group) = chunk_group_by_ukey.get(group_ukey) {
      // 获取子 chunk groups
      for child_group_ukey in group.children() {
        if let Some(child_group) = chunk_group_by_ukey.get(child_group_ukey) {
          // 获取子 group 中的 chunks
          for child_chunk_ukey in child_group.chunks() {
            // 使用 ukey 作为 ID，确保与 chunks 列表一致
            children.push(child_chunk_ukey.as_u32().to_string());
          }
        }
      }
    }
  }

  // 去重
  children.sort();
  children.dedup();
  children
}
```

### 重要说明

**ID 存储方式**：
- ✅ 使用 `ukey.as_u32().to_string()` 获取 chunk ID
- ❌ 不使用 `chunk.id()`（可能为 None 导致空字符串）
- 这确保 `children` 中的 ID 能在外层 `chunks` 列表中正确匹配

**存储内容**：
- 存储**子 chunk 的 ID 列表**
- 表示当前 chunk 引用/依赖了哪些 chunk（通常是异步加载的）
- 例如：入口 chunk `main` 通过 `import()` 异步加载了 `lazy-route`，则 `main.children` 包含 `lazy-route` 的 ID

### 增加的数据量

- **每个 Chunk**:
  - 叶子 chunk（无异步导入）: 0 个 children，0 字节
  - 包含异步导入的 chunk: 1-10 个 children，约 20-200 字节
  - 路由 chunk（大量懒加载）: 可能 10+ 个 children
- **典型项目** (20 个 chunks，平均 2 个 children): 约 800 字节
- **大型项目** (200 个 chunks): 约 8KB
- **增长率**: 对总数据量影响 < 0.5%

### 性能开销

- **采集阶段**: 低
  - 与 parents 一起收集，几乎无额外开销
  - 时间复杂度: O(n)
  - 典型开销: < 5ms
- **内存开销**: 低
- **传输开销**: 可忽略

### 可实现功能列表

1. **异步加载分析**:
   - 识别所有异步子模块
   - 评估代码拆分效果
2. **懒加载可视化**:
   - 展示动态 import() 产生的 chunk
   - 路由懒加载关系图
3. **Bundle 优化建议**:
   - 识别过度拆分的情况
   - 建议合并小型异步 chunk
4. **加载性能优化**:
   - 评估异步 chunk 的加载时机
   - 建议 prefetch 优先级
5. **代码拆分效果评估**:
   - 统计首屏 vs 异步代码比例
   - 评估拆分粒度是否合理
6. **路由级别分析**:
   - 识别每个路由对应的 chunk 树
   - 优化路由级别的代码拆分

---

## 总结

### 建议实施顺序

**parents + children** (中优先级)
- 建议一起实现，完善整体依赖分析图景
- 实现成本低，数据增长小
- 建议在第二阶段实现

### 性能对比

| 字段     | 采集开销 | 数据增长 | 传输影响 |
| -------- | -------- | -------- | -------- |
| parents  | < 5ms    | < 0.5%   | 低       |
| children | < 5ms    | < 0.5%   | 低       |

### 重要提示

Chunk 的 parents 和 children 字段提供了 chunk 级别的依赖关系视图，与 Module 的依赖关系互补。建议在实现 Module 依赖关系后作为补充功能实现。

---

## 使用示例

### 实际场景示例

假设有以下代码结构：

```javascript
// main.js (入口文件)
import './common.js'
import('./lazy-page-a.js')  // 异步加载页面 A
import('./lazy-page-b.js')  // 异步加载页面 B

// lazy-page-a.js
import('./shared-component.js')  // 异步加载共享组件

// lazy-page-b.js
import('./shared-component.js')  // 异步加载共享组件
```

### Chunk 依赖关系

打包后生成 4 个 chunks，ID 分别为：

```json
{
  "chunks": [
    {
      "id": "1",           // main chunk
      "names": ["main"],
      "parents": [],
      "children": ["2", "3"]
    },
    {
      "id": "2",           // lazy-page-a chunk
      "names": ["lazy-page-a"],
      "parents": ["1"],
      "children": ["4"]
    },
    {
      "id": "3",           // lazy-page-b chunk
      "names": ["lazy-page-b"],
      "parents": ["1"],
      "children": ["4"]
    },
    {
      "id": "4",           // shared-component chunk
      "names": ["shared-component"],
      "parents": ["2", "3"],
      "children": []
    }
  ]
}
```

### 依赖关系可视化

```
┌──────────┐
│ main (1) │ ← 入口 chunk，没有 parents
└────┬─────┘
     │
     ├─────────────┬─────────────┐
     ↓             ↓             │
┌─────────────┐ ┌─────────────┐ │
│page-a (2)   │ │page-b (3)   │ │
│parents: [1] │ │parents: [1] │ │
└──────┬──────┘ └──────┬──────┘ │
       │               │         │
       └───────┬───────┘         │
               ↓                 │
       ┌────────────────┐        │
       │ shared (4)     │ ←──────┘
       │ parents: [2,3] │
       │ children: []   │ ← 叶子节点
       └────────────────┘
```

### 前端使用示例

```typescript
// 1. 查找某个 chunk 的所有祖先（加载路径）
function getAncestors(chunkId: string, chunks: Chunk[]): string[] {
  const chunk = chunks.find(c => c.id === chunkId);
  if (!chunk || !chunk.parents.length) return [];

  const ancestors = [...chunk.parents];
  chunk.parents.forEach(parentId => {
    ancestors.push(...getAncestors(parentId, chunks));
  });

  return [...new Set(ancestors)];
}

// 例：getAncestors("4", chunks) → ["2", "3", "1"]
// 表示加载 shared-component 需要先加载 main, page-a/page-b


// 2. 查找某个 chunk 的所有后代（依赖树）
function getDescendants(chunkId: string, chunks: Chunk[]): string[] {
  const chunk = chunks.find(c => c.id === chunkId);
  if (!chunk || !chunk.children.length) return [];

  const descendants = [...chunk.children];
  chunk.children.forEach(childId => {
    descendants.push(...getDescendants(childId, chunks));
  });

  return [...new Set(descendants)];
}

// 例：getDescendants("1", chunks) → ["2", "3", "4"]
// 表示 main chunk 会触发加载所有其他 chunks


// 3. 分析关键路径（critical path）
function getCriticalPath(chunkId: string, chunks: Chunk[]): string[] {
  const ancestors = getAncestors(chunkId, chunks);
  return [chunkId, ...ancestors].reverse();
}

// 例：getCriticalPath("4", chunks) → ["1", "2", "4"] 或 ["1", "3", "4"]
// 表示加载 shared-component 的完整路径


// 4. 检测重复依赖
function findSharedDependencies(chunks: Chunk[]): Map<string, string[]> {
  const sharedMap = new Map<string, string[]>();

  chunks.forEach(chunk => {
    if (chunk.parents.length > 1) {
      sharedMap.set(chunk.id, chunk.parents);
    }
  });

  return sharedMap;
}

// 例：findSharedDependencies(chunks) → Map { "4" => ["2", "3"] }
// 表示 chunk 4 被多个 chunk 共享，可能适合提取到 vendor
```

### 优化建议场景

基于 parents 和 children 关系，可以提供：

1. **过度拆分检测**：
   - 如果某个 chunk 的 children 数量 > 10，建议合并

2. **共享 chunk 优化**：
   - 如果某个 chunk 的 parents 数量 > 2，建议提取到 common chunk

3. **加载链分析**：
   - 计算从入口到目标 chunk 的最短路径长度
   - 路径过长（depth > 3）建议扁平化

4. **预加载建议**：
   - 分析高频访问的 children，建议添加 `<link rel="prefetch">`
