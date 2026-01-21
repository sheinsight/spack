# Bundle Analyzer 数据结构：为什么是图而不是树

## TL;DR

Bundle Analyzer 的数据结构是 **有向无环图（DAG）**，而不是树，主要原因是：

1. **Module → Chunk 是多对多关系**（共享模块）
2. **Chunk 之间有复杂的父子关系**（多个父节点）
3. **Package 层面也存在共享**（同一个包的模块可能分散在不同 chunk）

---

## 1. 树 vs 图的对比

### 1.1 树的特征
```
树（Tree）:
- 每个节点只有一个父节点（除了根节点）
- 没有循环
- 节点之间只有一条路径

示例：
       A
      / \
     B   C
    / \
   D   E
```

### 1.2 图的特征
```
图（Graph）:
- 节点可以有多个父节点
- 可能存在循环（有向图）或无循环（DAG）
- 节点之间可能有多条路径

示例：
       A
      / \
     B   C
      \ / \
       D   E
     (D 有两个父节点 B 和 C)
```

---

## 2. 为什么 Bundle Analyzer 是图？

### 2.1 共享模块问题（Module → Chunk 多对多）

**场景**：公共库被多个页面引用

```javascript
// page-a.js
import React from 'react';
import { Header } from './components/Header';

// page-b.js
import React from 'react';
import { Footer } from './components/Footer';
```

**打包结果**（假设没有 splitChunks）：

```
Package: react
   │
   └─ Module: react/index.js
        │
        ├─ Chunk: page-a.chunk  ←┐
        │                         ├─ 同一个 Module 属于两个 Chunk！
        └─ Chunk: page-b.chunk  ←┘

如果是树结构，react/index.js 只能属于一个 Chunk，但实际上它被两个 Chunk 共享。
```

**图示**：

```
         [react Package]
               |
               v
      [react/index.js Module]
            /     \
           /       \
          v         v
   [page-a.chunk] [page-b.chunk]
```

这是典型的 **DAG** 结构，不是树！

---

### 2.2 Chunk 的多父节点问题

**场景**：多个路由页面都异步加载同一个组件

```javascript
// route-a.js
const SharedComponent = () => import('./SharedComponent');

// route-b.js
const SharedComponent = () => import('./SharedComponent');

// route-c.js
const SharedComponent = () => import('./SharedComponent');
```

**打包结果**：

```
[main.chunk]
    |
    ├─ [route-a.chunk] ─┐
    |                    |
    ├─ [route-b.chunk] ─┼─→ [shared-component.chunk]
    |                    |
    └─ [route-c.chunk] ─┘

shared-component.chunk 有 3 个 parents！
```

**Chunk 关系图**：

```
      [main]
      /  |  \
     /   |   \
    v    v    v
  [A]  [B]  [C]
    \   |   /
     \  |  /
      v v v
    [Shared]  ← 有 3 个父节点
```

如果是树，每个节点只能有一个父节点，这里明显不符合！

---

### 2.3 实际项目中的复杂图结构

**真实的 React + React Router 应用**：

```
Packages:
  react ──────┬─→ Module: react/index.js ───┬─→ Chunk: main
              │                               ├─→ Chunk: vendor
              └─→ Module: react/jsx-runtime ─┤
                                              └─→ Chunk: page-a

Chunks:
  main (entry) ──────┬─→ page-a
                     ├─→ page-b
                     └─→ shared-utils

  page-a ────────────┬─→ shared-utils ←─┐
                     └─→ ui-components ←─┼─ 这两个 chunk
                                         │   都被多个父节点引用
  page-b ────────────┬─→ shared-utils ←─┘
                     └─→ ui-components

  vendor (shared) ───→ (被 main 引用)
```

**可视化**：

```
              [react Package]
              /            \
             /              \
            v                v
  [react/index.js]   [react/jsx-runtime]
      /    |    \          /    \
     /     |     \        /      \
    v      v      v      v        v
[main] [vendor] [page-a] [page-b] [page-c]
    \     /        \      /
     \   /          \    /
      v v            v  v
  [shared-utils]  [ui-components]
```

这是典型的 **多对多关系图**，绝对不是树！

---

## 3. 数学证明

### 3.1 树的定义
- 有 `n` 个节点的树有且仅有 `n-1` 条边
- 任意两个节点之间有且仅有一条路径

### 3.2 Bundle Analyzer 不满足树的条件

**反例 1：边数超过 n-1**

```
4 个 Modules: M1, M2, M3, M4
3 个 Chunks: C1, C2, C3

关系：
M1 → C1
M2 → C1, C2  (M2 属于两个 Chunk)
M3 → C2, C3  (M3 属于两个 Chunk)
M4 → C3

总共 7 条边，但如果是树，4+3=7 个节点应该只有 6 条边。
```

**反例 2：存在多条路径**

```
从 Package react 到 Asset main.js 有多条路径：

Path 1: react → react/index.js → main.chunk → main.js
Path 2: react → react/index.js → vendor.chunk → vendor.js
                                      ↓
                              (vendor.chunk 也被 main.js 引用)

这违反了树的"任意两点只有一条路径"的定义。
```

---

## 4. 图的类型

Bundle Analyzer 是 **有向无环图（DAG, Directed Acyclic Graph）**：

### 4.1 为什么是有向图？
- Package → Module → Chunk → Asset 有明确的方向
- Chunk.parents → Chunk → Chunk.children 有明确的方向

### 4.2 为什么是无环的？
- 虽然 Module 依赖可能有循环（A 依赖 B，B 依赖 A）
- 但 rspack 在打包时会处理循环依赖，最终的 Chunk 关系是无环的
- Chunk 的 parents/children 关系不会形成循环（父 chunk 不会依赖子 chunk）

### 4.3 DAG 的特性
```
✅ 可以拓扑排序（确定加载顺序）
✅ 可以计算关键路径（性能优化）
✅ 可以并行处理（多个分支独立）
❌ 不能简单地用树遍历算法
```

---

## 5. 不同层级的结构

### 5.1 Package 层级：森林（Forest）

**单独看 Package → Module 关系**，可以看作多棵树的集合：

```
[react Package]       [lodash Package]      [your-app Package]
       |                     |                       |
       ├─ react/index        ├─ lodash/map          ├─ src/index
       ├─ react/jsx-runtime  ├─ lodash/filter       ├─ src/app
       └─ react/...          └─ lodash/...          └─ src/...

每个 Package 内部是树形结构，但整体是森林（多棵树）。
```

### 5.2 Module → Chunk：多对多图（Graph）

**这是最复杂的关系**，不是树：

```
Modules:                Chunks:
  M1 ────────┬────────→ C1
             │
  M2 ────────┼────┬───→ C2
             │    │
  M3 ────────┴────┴───→ C3
```

### 5.3 Chunk → Asset：可能是树，也可能是图

**情况 1：一对一映射（树形）**
```
Chunk: main ──→ Asset: main.js
Chunk: vendor ──→ Asset: vendor.js
```

**情况 2：多个输出（仍然是树，但有多个子节点）**
```
Chunk: main ──┬→ Asset: main.js
              ├→ Asset: main.css
              └→ Asset: main.js.map
```

**情况 3：代码分割后的共享（可能是图）**
```
某些高级配置下，多个 Chunk 可能影响同一个 Asset 的生成
（较少见，通常不会这样配置）
```

---

## 6. 如何遍历这个图结构

### 6.1 不能用树遍历算法

❌ **错误的树遍历**：
```typescript
function traverse(node) {
  visit(node);
  for (const child of node.children) {
    traverse(child);  // 可能重复访问同一个节点！
  }
}
```

### 6.2 需要用图遍历算法

✅ **正确的图遍历**：
```typescript
function traverse(startNode) {
  const visited = new Set();
  const queue = [startNode];

  while (queue.length > 0) {
    const node = queue.shift();

    if (visited.has(node.id)) {
      continue;  // 已访问过，跳过
    }

    visited.add(node.id);
    visit(node);

    for (const child of node.children) {
      if (!visited.has(child.id)) {
        queue.push(child);
      }
    }
  }
}
```

### 6.3 拓扑排序（确定加载顺序）

```typescript
// 计算 Chunk 加载顺序
function topologicalSort(chunks: Chunk[]): Chunk[] {
  const sorted = [];
  const visited = new Set();
  const temp = new Set();  // 检测循环

  function visit(chunkId: string) {
    if (temp.has(chunkId)) {
      throw new Error('Circular dependency detected!');
    }
    if (visited.has(chunkId)) {
      return;
    }

    temp.add(chunkId);

    const chunk = chunks.find(c => c.id === chunkId);
    for (const parentId of chunk.parents) {
      visit(parentId);  // 先访问依赖
    }

    temp.delete(chunkId);
    visited.add(chunkId);
    sorted.push(chunk);
  }

  // 从入口 chunk 开始
  const entryChunks = chunks.filter(c => c.entry);
  for (const entry of entryChunks) {
    visit(entry.id);
  }

  return sorted;
}
```

---

## 7. 实际分析场景

### 7.1 查找所有依赖路径（DFS）

```typescript
// 从 Package 到 Asset 的所有路径
function findAllPaths(
  packageName: string,
  assetName: string
): Path[] {
  const paths = [];

  function dfs(currentPath: string[], visited: Set<string>) {
    const last = currentPath[currentPath.length - 1];

    if (isAsset(last) && last === assetName) {
      paths.push([...currentPath]);
      return;
    }

    for (const next of getNeighbors(last)) {
      if (!visited.has(next)) {
        visited.add(next);
        dfs([...currentPath, next], visited);
        visited.delete(next);  // 回溯
      }
    }
  }

  const pkg = packages.find(p => p.name === packageName);
  dfs([pkg.name], new Set([pkg.name]));

  return paths;
}

// 示例结果：
// react → react/index.js → main.chunk → main.js
// react → react/jsx-runtime → vendor.chunk → vendor.js → main.js (引用)
```

### 7.2 计算影响范围（BFS）

```typescript
// 如果修改某个 Module，会影响哪些 Assets？
function calculateImpact(moduleId: string): string[] {
  const affectedAssets = new Set<string>();
  const queue = [moduleId];
  const visited = new Set([moduleId]);

  while (queue.length > 0) {
    const current = queue.shift();

    if (isAsset(current)) {
      affectedAssets.add(current);
      continue;
    }

    // 向下游传播
    for (const next of getDownstreamNodes(current)) {
      if (!visited.has(next)) {
        visited.add(next);
        queue.push(next);
      }
    }
  }

  return Array.from(affectedAssets);
}
```

### 7.3 查找共享模块

```typescript
// 找出被多个 Chunk 共享的 Module
function findSharedModules(modules: Module[]): Module[] {
  return modules.filter(m => m.chunks.length > 1);
}

// 找出共享最多的模块（优化候选）
function findMostSharedModules(modules: Module[], topN: number): Module[] {
  return modules
    .filter(m => m.chunks.length > 1)
    .sort((a, b) => b.chunks.length - a.chunks.length)
    .slice(0, topN);
}
```

---

## 8. 图算法应用

### 8.1 最短路径（Dijkstra）

```typescript
// 找出从入口到目标 Module 的最短加载路径
function shortestLoadPath(targetModuleId: string): Chunk[] {
  // 使用 Dijkstra 算法，权重为 Chunk 大小
  // 目标：最小化总下载大小
}
```

### 8.2 关键路径（Critical Path）

```typescript
// 找出加载时间最长的依赖链
function criticalPath(): Chunk[] {
  // 类似 CPM（关键路径方法）
  // 权重为 Chunk 加载时间
}
```

### 8.3 社区检测（Community Detection）

```typescript
// 识别紧密耦合的 Module 群
// 建议将它们打包到同一个 Chunk
function detectModuleCommunities(): Module[][] {
  // 使用 Louvain 算法或 Label Propagation
}
```

---

## 9. 可视化方案

### 9.1 力导向图（Force-Directed Graph）

最适合展示复杂的多对多关系：

```
D3.js force simulation:
- 节点：Package/Module/Chunk/Asset
- 边：依赖关系
- 力：排斥力（避免重叠）+ 吸引力（相关节点靠近）
```

### 9.2 桑基图（Sankey Diagram）

展示数据流向和大小：

```
Package (总大小) ───→ Module (模块大小) ───→ Chunk (块大小) ───→ Asset (文件大小)
   react (50KB)      ├─ index: 30KB          ├─ main: 80KB      ├─ main.js: 100KB
                      └─ jsx: 20KB            └─ vendor: 40KB    └─ vendor.js: 50KB
```

### 9.3 树状图（Treemap）+ 连线

- Treemap 展示 Package/Module 的大小比例
- 连线展示 Module → Chunk 的多对多关系

---

## 总结

| 特性               | 树（Tree） | Bundle Analyzer |
| ------------------ | ---------- | --------------- |
| 每个节点的父节点数 | 1          | **多个**        |
| 节点间路径数量     | 1          | **多条**        |
| 边数（n 节点）     | n-1        | **> n-1**       |
| 数据结构           | Tree       | **DAG**         |
| 遍历算法           | DFS/BFS    | **图遍历（需要 visited）** |
| 适合的可视化       | TreeMap    | **Force Graph / Sankey** |

**结论**：

1. ❌ Bundle Analyzer **不是树**
2. ✅ 是 **有向无环图（DAG）**
3. 🔑 关键原因：**Module → Chunk 多对多关系**
4. 📊 需要用**图算法**分析和可视化
5. 🎯 这正是 Bundle Analyzer 强大的原因：可以分析复杂的模块共享和依赖关系！
