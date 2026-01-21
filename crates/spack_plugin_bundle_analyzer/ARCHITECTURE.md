# Bundle Analyzer 架构：核心概念关系图

本文档详细说明 Bundle Analyzer 中四个核心概念的关系和定义。

## 核心概念层次结构

```
Package (npm 包)
    ↓ 包含
Module (源文件)
    ↓ 打包到
Chunk (代码块)
    ↓ 生成
Asset (输出文件)
```

---

## 1. 从上到下的视角：构建流程

### 1.1 Package → Module

**Package（npm 包）**：
- 定义：来自 node_modules 的第三方依赖包
- 示例：`react`, `lodash`, `@babel/core`
- 特征：
  - 有 package.json 文件
  - 有版本号
  - 包含多个模块文件

**关系**：
- 一个 Package **包含** 多个 Module
- 例如 `react` 包包含：
  - `react/index.js`
  - `react/jsx-runtime.js`
  - `react/jsx-dev-runtime.js`
  - 等等...

### 1.2 Module → Chunk

**Module（源文件/模块）**：
- 定义：编译过程中的最小单位，对应源代码中的单个文件
- 示例：
  - `./src/index.js` (用户代码)
  - `./node_modules/react/index.js` (第三方代码)
- 种类（ModuleKind）：
  - **Normal**: 普通模块（最常见）
  - **Concatenated**: 合并模块（Scope Hoisting 优化后）
  - **External**: 外部模块（通过 externals 配置排除的）
  - **Context**: 上下文模块（如 `require.context()`）
  - **Raw**: 原始模块（直接嵌入的字符串或Buffer）
  - **SelfRef**: 自引用模块

**Chunk（代码块）**：
- 定义：打包后的代码单元，包含一组相关的模块
- 类型：
  - **Entry Chunk**: 入口块（由配置的 entry 生成）
  - **Async Chunk**: 异步块（由 `import()` 动态导入生成）
  - **Split Chunk**: 分割块（由 splitChunks 配置生成）

**关系**：
- 一个 Module **可以属于多个** Chunk（如共享模块）
- 一个 Chunk **包含多个** Module
- 例如：
  ```
  main.chunk (entry=true)
    ├─ ./src/index.js
    ├─ ./src/utils.js
    └─ ./node_modules/react/index.js

  vendor.chunk (entry=false, initial=true)
    ├─ ./node_modules/react/index.js  ← 与 main 共享
    └─ ./node_modules/react-dom/index.js

  lazy-route.chunk (entry=false, initial=false)
    └─ ./src/pages/about.js
  ```

### 1.3 Chunk → Asset

**Asset（输出文件）**：
- 定义：最终写入磁盘的文件
- 示例：
  - `main.js` (JavaScript 输出)
  - `main.css` (CSS 输出)
  - `logo.png` (静态资源)
- 类型（AssetType）：
  - **JavaScript**: `.js`, `.mjs`
  - **CSS**: `.css`
  - **HTML**: `.html`
  - **Image**: `.png`, `.jpg`, `.svg` 等
  - **Font**: `.woff`, `.ttf` 等
  - **Other**: 其他类型

**关系**：
- 一个 Chunk **通常生成一个或多个** Asset
- 例如：
  ```
  main.chunk
    ├─ main.js (主代码)
    ├─ main.css (提取的样式)
    └─ main.js.map (source map)
  ```

---

## 2. 从下到上的视角：分析流程

### 2.1 Asset ← Chunk

**问题：这个输出文件来自哪些代码块？**

```json
{
  "asset": {
    "name": "main.js",
    "size": 102400,
    "chunks": ["1", "2"]  // 关联的 chunk IDs
  }
}
```

### 2.2 Chunk ← Module

**问题：这个代码块包含哪些源文件？**

```json
{
  "chunk": {
    "id": "1",
    "names": ["main"],
    "modules": ["0", "1", "2", "3"],  // 包含的 module IDs
    "size": 50000
  }
}
```

### 2.3 Module ← Package

**问题：这个模块属于哪个 npm 包？**

```json
{
  "module": {
    "id": "1",
    "name": "./node_modules/react/index.js",
    "is_node_module": true,
    "name_for_condition": "/path/to/node_modules/react/index.js"
  },
  "package": {
    "name": "react",
    "version": "18.2.0",
    "modules": ["1", "5", "8"]  // 该包的所有模块 IDs
  }
}
```

---

## 3. 关键字段关系

### 3.1 ID 引用关系

所有对象通过 **字符串 ID** 相互引用：

```typescript
// Asset 引用 Chunk
Asset.chunks: string[]  // chunk IDs

// Chunk 引用 Module
Chunk.modules: string[]  // module IDs
Chunk.parents: string[]  // parent chunk IDs
Chunk.children: string[]  // child chunk IDs

// Package 引用 Module
Package.modules: string[]  // module IDs

// Module 引用 Chunk
Module.chunks: string[]  // chunk IDs
```

### 3.2 完整的数据流

```
用户代码 + node_modules
        ↓
    [编译阶段]
        ↓
   Module Graph (模块图)
        ↓
    [打包阶段]
        ↓
    Chunk Graph (代码块图)
        ↓
    [生成阶段]
        ↓
      Assets (输出文件)
```

---

## 4. 实际案例

### 4.1 简单的 React 应用

**源代码结构**：
```
src/
  ├─ index.js         // 入口文件，导入 react
  ├─ App.js           // 主组件
  └─ About.js         // 异步加载的页面

node_modules/
  └─ react/
      ├─ index.js
      └─ jsx-runtime.js
```

**编译后的关系**：

```json
{
  "packages": [
    {
      "name": "react",
      "version": "18.2.0",
      "modules": ["module_3", "module_4"],  // react 的模块
      "size": 50000
    }
  ],

  "modules": [
    {
      "id": "module_1",
      "name": "./src/index.js",
      "chunks": ["chunk_1"],
      "is_node_module": false
    },
    {
      "id": "module_2",
      "name": "./src/App.js",
      "chunks": ["chunk_1"],
      "is_node_module": false
    },
    {
      "id": "module_3",
      "name": "./node_modules/react/index.js",
      "chunks": ["chunk_1"],
      "is_node_module": true
    },
    {
      "id": "module_5",
      "name": "./src/About.js",
      "chunks": ["chunk_2"],  // 异步 chunk
      "is_node_module": false
    }
  ],

  "chunks": [
    {
      "id": "chunk_1",
      "names": ["main"],
      "modules": ["module_1", "module_2", "module_3"],
      "files": ["main.js"],
      "entry": true,
      "initial": true,
      "children": ["chunk_2"]  // 引用了异步 chunk
    },
    {
      "id": "chunk_2",
      "names": ["about"],
      "modules": ["module_5"],
      "files": ["about.js"],
      "entry": false,
      "initial": false,
      "parents": ["chunk_1"]  // 被 main chunk 引用
    }
  ],

  "assets": [
    {
      "name": "main.js",
      "size": 102400,
      "chunks": ["chunk_1"],  // 由 main chunk 生成
      "asset_type": "JavaScript"
    },
    {
      "name": "about.js",
      "size": 20480,
      "chunks": ["chunk_2"],  // 由 about chunk 生成
      "asset_type": "JavaScript"
    }
  ]
}
```

---

## 5. 常见分析场景

### 5.1 查找某个 npm 包的总大小

```typescript
// 1. 找到 Package
const reactPackage = packages.find(p => p.name === 'react');

// 2. 获取该包的所有模块
const reactModules = modules.filter(m =>
  reactPackage.modules.includes(m.id)
);

// 3. 计算总大小
const totalSize = reactModules.reduce((sum, m) => sum + m.size, 0);
```

### 5.2 分析某个输出文件的组成

```typescript
// 1. 找到 Asset
const mainAsset = assets.find(a => a.name === 'main.js');

// 2. 获取相关的 Chunks
const relatedChunks = chunks.filter(c =>
  mainAsset.chunks.includes(c.id)
);

// 3. 获取所有包含的 Modules
const allModuleIds = relatedChunks.flatMap(c => c.modules);
const allModules = modules.filter(m => allModuleIds.includes(m.id));

// 4. 按来源分组
const byPackage = allModules.reduce((acc, m) => {
  const pkg = packages.find(p => p.modules.includes(m.id));
  const key = pkg ? pkg.name : 'app';
  acc[key] = (acc[key] || 0) + m.size;
  return acc;
}, {});
```

### 5.3 追踪异步加载关系

```typescript
// 1. 找到入口 chunk
const entryChunk = chunks.find(c => c.entry);

// 2. 递归获取所有子 chunks
function getAllAsyncChunks(chunkId: string): string[] {
  const chunk = chunks.find(c => c.id === chunkId);
  if (!chunk || !chunk.children.length) return [];

  const children = chunk.children;
  const grandChildren = children.flatMap(getAllAsyncChunks);

  return [...children, ...grandChildren];
}

const allAsyncChunks = getAllAsyncChunks(entryChunk.id);
```

---

## 6. Module 类型详解

### 6.1 Normal Module（最常见）
- 普通的 JavaScript/TypeScript 文件
- 经过 loader 处理（如 babel-loader, ts-loader）
- 示例：`./src/utils.js`

### 6.2 Concatenated Module（优化后）
- 通过 Scope Hoisting 合并的模块
- 多个小模块合并成一个，减少函数调用开销
- `module.concatenated_modules` 字段包含被合并的模块列表
- 示例：
  ```json
  {
    "id": "concat_1",
    "module_kind": "Concatenated",
    "concatenated_modules": [
      { "id": "1", "name": "./src/a.js", "size": 100 },
      { "id": "2", "name": "./src/b.js", "size": 200 }
    ]
  }
  ```

### 6.3 External Module
- 配置为外部依赖的模块（不打包进 bundle）
- 示例：`externals: { react: 'React' }`
- 运行时从全局变量或 CDN 加载

### 6.4 Context Module
- 动态 require 生成的模块
- 示例：`require.context('./locales', true, /\.json$/)`
- 包含匹配目录下的所有文件

### 6.5 Raw Module
- 直接嵌入的原始内容
- 示例：`new webpack.BannerPlugin()` 生成的 banner 注释

### 6.6 SelfRef Module
- 模块的自引用（较少见）
- 用于处理循环依赖等特殊情况

---

## 7. 数据收集顺序

在 `BundleAnalyzerPlugin` 中，数据按以下顺序收集（见 `lib.rs:58-90`）：

```rust
1. ModuleChunkContext     // 预构建映射关系（性能优化）
   ↓
2. Assets                 // 输出文件
   ↓
3. Modules                // 源模块（使用预构建的映射）
   ↓
4. Chunks                 // 代码块（使用预构建的映射）
   ↓
5. Packages               // npm 包（聚合 Modules）
```

**为什么这个顺序？**
- ModuleChunkContext 提供 O(1) 查找，避免后续重复遍历
- Assets 最独立，不依赖其他数据
- Modules 需要查找所属的 Chunks
- Chunks 需要查找包含的 Modules
- Packages 需要聚合 Modules 数据

---

## 总结

```
┌─────────────────────────────────────────────────────┐
│                    Package                          │
│  (npm 包，如 react@18.2.0)                          │
│  - 包含多个 Module                                   │
│  - 有版本号                                          │
└──────────────────┬──────────────────────────────────┘
                   │ 包含
                   ↓
┌─────────────────────────────────────────────────────┐
│                    Module                           │
│  (源文件，如 ./src/index.js)                        │
│  - 编译的最小单位                                     │
│  - 有不同种类（Normal/Concatenated/External 等）     │
│  - 可属于多个 Chunk                                  │
└──────────────────┬──────────────────────────────────┘
                   │ 打包到
                   ↓
┌─────────────────────────────────────────────────────┐
│                    Chunk                            │
│  (代码块，如 main.chunk)                             │
│  - 包含多个 Module                                   │
│  - 有父子关系（parents/children）                    │
│  - 可以是入口/异步/分割块                             │
└──────────────────┬──────────────────────────────────┘
                   │ 生成
                   ↓
┌─────────────────────────────────────────────────────┐
│                    Asset                            │
│  (输出文件，如 main.js)                              │
│  - 最终写入磁盘的文件                                 │
│  - 有类型（JS/CSS/Image 等）                         │
│  - 可选压缩大小（gzip/brotli）                        │
└─────────────────────────────────────────────────────┘
```

**关键点**：
- **Package → Module**: 一对多（一个包包含多个模块）
- **Module → Chunk**: 多对多（模块可被多个 chunk 共享）
- **Chunk → Asset**: 一对多（一个 chunk 可生成多个文件）
- **通过 ID 引用**: 所有对象通过字符串 ID 相互关联
