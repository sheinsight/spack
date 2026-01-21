# Bundle Analyzer UI 设计：Chunk 局部树视图

## 核心观点

虽然整体数据结构是 **图（DAG）**，但在特定的 UI 交互场景下，**通过限定上下文可以将其简化为树**。

---

## 你的交互设计（树形结构）

### 交互流程

```
步骤 1: 展示 Chunks 列表
  ├─ main.chunk
  ├─ vendor.chunk
  └─ page-a.chunk

步骤 2: 用户点击 "main.chunk"
  ↓
  展开该 Chunk 下的所有 Modules（树形结构）

步骤 3: 如果 Module 是 Concatenated 类型
  ↓
  继续展开显示内部的 concatenated_modules
```

### 数据结构概览

```typescript
// 单个 Chunk 的树形视图
interface ChunkTreeView {
  chunk: Chunk;
  children: ModuleTreeNode[];
}

interface ModuleTreeNode {
  module: Module;
  children?: ModuleTreeNode[];  // 仅 Concatenated Module 有子节点
}
```

---

## 完整数据结构定义

### TypeScript 接口定义

```typescript
// ============================================
// 基础数据结构
// ============================================

/**
 * 输出文件
 */
interface Asset {
  // 文件名
  name: string;
  // 文件大小（字节）
  size: number;
  // gzip 压缩后的大小（可选）
  gzip_size?: number;
  // brotli 压缩后的大小（可选）
  brotli_size?: number;
  // 关联的 chunk IDs
  chunks: string[];
  // 是否已生成到磁盘
  emitted: boolean;
  // 文件类型
  asset_type: AssetType;
}

/**
 * 资源类型枚举
 */
type AssetType =
  | 'JavaScript'
  | 'CSS'
  | 'HTML'
  | 'Image'
  | 'Font'
  | 'Video'
  | 'Audio'
  | 'Other';

/**
 * 代码块
 */
interface Chunk {
  // chunk ID
  id: string;
  // chunk 名称列表
  names: string[];
  // chunk 总大小（字节）
  size: number;
  // 包含的模块 ID 列表
  modules: string[];
  // 是否入口 chunk
  entry: boolean;
  // 是否初始 chunk
  initial: boolean;
  // 是否包含异步 chunk
  async_chunks: boolean;
  // 是否包含运行时代码
  runtime: boolean;
  // chunk 创建原因
  reason: string;
  // chunk 生成的输出文件列表
  files: string[];
  // 父 chunk ID 列表
  parents: string[];
  // 子 chunk ID 列表
  children: string[];
}

/**
 * 模块
 */
interface Module {
  // 模块唯一 ID
  id: string;
  // 可读名称，如 "./src/index.js"
  name: string;
  // 模块大小（字节）
  size: number;
  // 包含此模块的 chunk IDs
  chunks: string[];
  // 模块种类
  module_kind: ModuleKind;
  // 模块文件类型
  module_type: ModuleType;
  // 是否来自 node_modules
  is_node_module: boolean;
  // 模块条件名称（用于模块解析）
  name_for_condition: string;
  // 合并的模块列表（如果是 ConcatenatedModule）
  concatenated_modules?: ConcatenatedModuleInfo[];
}

/**
 * 模块种类枚举
 */
type ModuleKind =
  | 'Normal'        // 普通模块
  | 'Concatenated'  // 合并模块（Scope Hoisting）
  | 'External'      // 外部模块
  | 'Context'       // 上下文模块（require.context）
  | 'Raw'           // 原始模块
  | 'SelfRef';      // 自引用模块

/**
 * 模块类型枚举
 */
type ModuleType =
  | 'JavaScript'
  | 'TypeScript'
  | 'JSX'
  | 'TSX'
  | 'CSS'
  | 'SCSS'
  | 'LESS'
  | 'JSON'
  | 'WebAssembly'
  | 'Asset'
  | 'Unknown';

/**
 * 合并模块中的单个内部模块信息
 */
interface ConcatenatedModuleInfo {
  // 模块 ID
  id: string;
  // 模块名称
  name: string;
  // 模块大小（字节）
  size: number;
}

/**
 * npm 包
 */
interface Package {
  // 包名，如 "react" 或 "@babel/core"
  name: string;
  // 版本号
  version: string;
  // 该包的总大小（字节）
  size: number;
  // 包含的模块数量
  module_count: number;
  // 该包包含的所有模块 ID 列表
  modules: string[];
  // package.json 文件路径
  package_json_path: string;
}

// ============================================
// 树视图专用数据结构
// ============================================

/**
 * 单个 Chunk 的树形视图
 */
interface ChunkTreeView {
  // Chunk 信息
  chunk: Chunk;
  // 模块树节点列表
  children: ModuleTreeNode[];
}

/**
 * 模块树节点
 */
interface ModuleTreeNode {
  // 模块信息
  module: Module;
  // 子节点（仅 Concatenated Module 有）
  children?: ModuleTreeNode[];
}

// ============================================
// 列表视图专用数据结构
// ============================================

/**
 * Chunk 列表项（用于首页展示）
 */
interface ChunkListItem {
  // Chunk ID
  id: string;
  // Chunk 名称列表
  names: string[];
  // 总大小（字节）
  size: number;
  // 包含的模块数量
  module_count: number;
  // 关联的输出文件
  output_files: OutputFile[];
  // 标签
  badges: ChunkBadge[];
}

/**
 * 输出文件简要信息
 */
interface OutputFile {
  // 文件名
  name: string;
  // 文件大小（字节）
  size: number;
  // 文件类型
  type: AssetType;
}

/**
 * Chunk 标签
 */
type ChunkBadge =
  | 'Entry'
  | 'Initial'
  | 'Async'
  | 'Runtime';

// ============================================
// 统计信息
// ============================================

/**
 * 性能计时信息
 */
interface PerformanceTimings {
  // 收集 Assets 耗时（毫秒）
  collect_assets_ms: number;
  // 收集 Modules 耗时（毫秒）
  collect_modules_ms: number;
  // 收集 Chunks 耗时（毫秒）
  collect_chunks_ms: number;
  // 分析 Packages 耗时（毫秒）
  analyze_packages_ms: number;
  // 总耗时（毫秒）
  total_ms: number;
}

/**
 * 汇总信息
 */
interface Summary {
  // 总大小（字节）
  total_size: number;
  // gzip 压缩后总大小（字节）
  total_gzip_size: number;
  // Assets 总数
  total_assets: number;
  // Modules 总数
  total_modules: number;
  // Chunks 总数
  total_chunks: number;
  // 构建耗时（毫秒）
  build_time: number;
  // 性能计时详情
  timings: PerformanceTimings;
}

/**
 * 完整报告
 */
interface Report {
  // 时间戳（Unix 毫秒）
  timestamp: number;
  // 汇总信息
  summary: Summary;
  // 所有 Assets
  assets: Asset[];
  // 所有 Modules
  modules: Module[];
  // 所有 Chunks
  chunks: Chunk[];
  // 所有 Packages
  packages: Package[];
}
```

### Rust Struct 定义

```rust
use napi_derive::napi;

// ============================================
// 基础数据结构
// ============================================

/// 输出文件
#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsAsset {
  pub name: String,
  pub size: u32,
  pub gzip_size: Option<u32>,
  pub brotli_size: Option<u32>,
  pub chunks: Vec<String>,
  pub emitted: bool,
  pub asset_type: String,
}

/// 代码块
#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsChunk {
  pub id: String,
  pub names: Vec<String>,
  pub size: u32,
  pub modules: Vec<String>,
  pub entry: bool,
  pub initial: bool,
  pub async_chunks: bool,
  pub runtime: bool,
  pub reason: String,
  pub files: Vec<String>,
  pub parents: Vec<String>,
  pub children: Vec<String>,
}

/// 模块
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
}

/// 合并模块中的单个内部模块信息
#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsConcatenatedModuleInfo {
  pub id: String,
  pub name: String,
  pub size: u32,
}

/// npm 包
#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsPackage {
  pub name: String,
  pub version: String,
  pub size: u32,
  pub module_count: u32,
  pub modules: Vec<String>,
  pub package_json_path: String,
}

// ============================================
// 树视图专用数据结构
// ============================================

/// 单个 Chunk 的树形视图
#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsChunkTreeView {
  pub chunk: JsChunk,
  pub children: Vec<JsModuleTreeNode>,
}

/// 模块树节点
#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsModuleTreeNode {
  pub module: JsModule,
  pub children: Option<Vec<JsModuleTreeNode>>,
}

// ============================================
// 列表视图专用数据结构
// ============================================

/// Chunk 列表项（用于首页展示）
#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsChunkListItem {
  pub id: String,
  pub names: Vec<String>,
  pub size: u32,
  pub module_count: u32,
  pub output_files: Vec<JsOutputFile>,
  pub badges: Vec<String>,
}

/// 输出文件简要信息
#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsOutputFile {
  pub name: String,
  pub size: u32,
  pub r#type: String,  // 'type' 是 Rust 关键字，用 r#type
}

// ============================================
// 统计信息
// ============================================

/// 性能计时信息
#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsPerformanceTimings {
  pub collect_assets_ms: f64,
  pub collect_modules_ms: f64,
  pub collect_chunks_ms: f64,
  pub analyze_packages_ms: f64,
  pub total_ms: f64,
}

/// 汇总信息
#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsSummary {
  pub total_size: u32,
  pub total_gzip_size: u32,
  pub total_assets: u32,
  pub total_modules: u32,
  pub total_chunks: u32,
  pub build_time: f64,
  pub timings: JsPerformanceTimings,
}

/// 完整报告
#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsBundleAnalyzerPluginResp {
  pub timestamp: u32,
  pub summary: JsSummary,
  pub assets: Vec<JsAsset>,
  pub modules: Vec<JsModule>,
  pub chunks: Vec<JsChunk>,
  pub packages: Vec<JsPackage>,
}
```

### 树形结构示例

```
main.chunk (root)
  ├─ Module: ./src/index.js
  ├─ Module: ./src/app.js
  ├─ Module: ./node_modules/react/index.js
  └─ Module: ./src/utils.js (Concatenated) ← 可以展开
       ├─ Inner: ./src/utils/format.js
       ├─ Inner: ./src/utils/validate.js
       └─ Inner: ./src/utils/helper.js
```

---

## 为什么这是树？

### 关键原因：限定了上下文

1. **选定了单个 Chunk 作为根节点**
   - 不会跨 Chunk 展示
   - 避免了多对多关系

2. **每个 Module 在当前视图中只出现一次**
   - 虽然 `react/index.js` 可能属于多个 Chunk
   - 但在 `main.chunk` 的视图中只显示一次

3. **Concatenated Module 的内部结构是天然的树**
   - 内部模块不会被其他 Module 共享
   - 形成严格的父子关系

### 对比：整体 vs 局部

```
整体视图（图结构）：
  [react/index.js Module]
        |
        ├──→ [main.chunk]
        └──→ [vendor.chunk]  ← 一个 Module 属于多个 Chunk

局部视图（树结构）：
  [main.chunk]  ← 选定这个 Chunk
     ├─ [react/index.js]  ← 只在这里显示一次
     ├─ [./src/app.js]
     └─ ...
```

---

## 实现方案

### 方案 1: 前端动态构建树

```typescript
// 1. 用户点击 Chunk
function buildChunkTree(chunkId: string): ChunkTreeView {
  const chunk = chunks.find(c => c.id === chunkId);

  // 2. 获取该 Chunk 的所有 Modules
  const chunkModules = modules.filter(m =>
    m.chunks.includes(chunkId)
  );

  // 3. 构建树节点
  const children = chunkModules.map(module =>
    buildModuleNode(module)
  );

  return { chunk, children };
}

// 递归构建 Module 节点（处理 Concatenated）
function buildModuleNode(module: Module): ModuleTreeNode {
  const node: ModuleTreeNode = {
    module,
    children: undefined
  };

  // 如果是 Concatenated Module，递归构建子节点
  if (module.module_kind === 'Concatenated' &&
      module.concatenated_modules) {
    node.children = module.concatenated_modules.map(inner => ({
      module: {
        id: inner.id,
        name: inner.name,
        size: inner.size,
        // 内部模块的简化信息
      } as Module,
      children: undefined  // 内部模块不再嵌套
    }));
  }

  return node;
}
```

### 方案 2: 后端预构建树结构

```rust
// 在 Rust 端生成树形结构

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsChunkTreeView {
  pub chunk: JsChunk,
  pub module_tree: Vec<JsModuleTreeNode>,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsModuleTreeNode {
  pub module: JsModule,
  pub children: Option<Vec<JsModuleTreeNode>>,
}

// 生成树的函数
impl Chunks {
  pub fn to_tree_view(&self, modules: &Modules) -> Vec<JsChunkTreeView> {
    self.0.iter().map(|chunk| {
      let chunk_modules: Vec<_> = modules
        .iter()
        .filter(|m| m.chunks.contains(&chunk.id))
        .collect();

      let module_tree = chunk_modules
        .into_iter()
        .map(|module| build_module_tree_node(module))
        .collect();

      JsChunkTreeView {
        chunk: chunk.clone().into(),
        module_tree,
      }
    }).collect()
  }
}
```

---

## UI 组件实现

### React 树形组件

```tsx
// ChunkTreeView.tsx
interface Props {
  chunkId: string;
}

export function ChunkTreeView({ chunkId }: Props) {
  const tree = useMemo(() =>
    buildChunkTree(chunkId),
    [chunkId]
  );

  return (
    <div className="chunk-tree">
      <ChunkHeader chunk={tree.chunk} />
      <div className="module-list">
        {tree.children.map(node => (
          <ModuleTreeNode
            key={node.module.id}
            node={node}
            depth={0}
          />
        ))}
      </div>
    </div>
  );
}

// ModuleTreeNode.tsx
interface ModuleTreeNodeProps {
  node: ModuleTreeNode;
  depth: number;
}

export function ModuleTreeNode({ node, depth }: ModuleTreeNodeProps) {
  const [expanded, setExpanded] = useState(false);
  const hasChildren = node.children && node.children.length > 0;

  return (
    <div
      className="module-node"
      style={{ paddingLeft: `${depth * 20}px` }}
    >
      <div className="module-info">
        {hasChildren && (
          <button onClick={() => setExpanded(!expanded)}>
            {expanded ? '▼' : '▶'}
          </button>
        )}

        <span className="module-name">{node.module.name}</span>
        <span className="module-size">{formatSize(node.module.size)}</span>

        {node.module.module_kind === 'Concatenated' && (
          <span className="badge">Concatenated</span>
        )}
      </div>

      {/* 展开 Concatenated Module 的内部模块 */}
      {expanded && hasChildren && (
        <div className="concatenated-modules">
          {node.children.map((child, idx) => (
            <ModuleTreeNode
              key={`${node.module.id}-${idx}`}
              node={child}
              depth={depth + 1}
            />
          ))}
        </div>
      )}
    </div>
  );
}
```

---

## 树形结构的优势

### 1. 简单直观
```
用户只需要理解：
  Chunk
    └─ Module
         └─ Inner Module (如果是 Concatenated)
```

### 2. 避免认知负担
- 不需要理解复杂的多对多关系
- 每次只关注一个 Chunk 的内容

### 3. 性能更好
- 不需要渲染整个图结构
- 可以懒加载（按 Chunk 加载）

### 4. 适合常见场景
- 用户通常关心："这个 chunk 包含了什么？"
- 而不是："所有 chunk 之间的关系是什么？"

---

## 扩展：跨 Chunk 的关系

如果需要展示跨 Chunk 的关系（比如"这个 Module 还属于哪些 Chunk"），可以：

### 方案 1: 显示提示信息

```tsx
<div className="module-info">
  <span className="module-name">{node.module.name}</span>

  {node.module.chunks.length > 1 && (
    <Tooltip content={`Also in: ${getOtherChunks(node.module)}`}>
      <span className="shared-badge">
        Shared ({node.module.chunks.length} chunks)
      </span>
    </Tooltip>
  )}
</div>
```

### 方案 2: 点击跳转到图视图

```tsx
<button onClick={() => showInGraphView(node.module.id)}>
  View in Graph
</button>
```

### 方案 3: Mini Graph 预览

```tsx
// 在树节点旁边显示小型关系图
<div className="module-node">
  <ModuleInfo module={node.module} />

  {node.module.chunks.length > 1 && (
    <MiniGraphPreview
      moduleId={node.module.id}
      relatedChunks={node.module.chunks}
    />
  )}
</div>
```

---

## 两种视图模式对比

### 树视图（你的方案）

**优点**：
- ✅ 简单直观
- ✅ 性能好（局部渲染）
- ✅ 符合用户心智模型
- ✅ 易于实现

**缺点**：
- ❌ 看不到跨 Chunk 关系
- ❌ 可能遗漏共享模块的信息

**适用场景**：
- 分析单个 Chunk 的组成
- 快速查看 Chunk 包含哪些模块
- 评估 Concatenated Module 的效果

### 图视图

**优点**：
- ✅ 完整展示所有关系
- ✅ 可以发现共享模块
- ✅ 适合优化分析

**缺点**：
- ❌ 复杂，学习曲线陡峭
- ❌ 性能开销大
- ❌ 可能信息过载

**适用场景**：
- 整体架构分析
- 共享模块优化
- 依赖关系诊断

---

## 推荐的 UI 架构

### 双模式切换

```tsx
function BundleAnalyzer() {
  const [viewMode, setViewMode] = useState<'tree' | 'graph'>('tree');

  return (
    <div>
      <ViewModeSwitch
        mode={viewMode}
        onChange={setViewMode}
      />

      {viewMode === 'tree' ? (
        <ChunkListView />  // 你的方案
      ) : (
        <GraphView />      // 完整图视图
      )}
    </div>
  );
}
```

### 默认树视图 + 可选图视图

```tsx
function ChunkDetail({ chunkId }: Props) {
  return (
    <div>
      {/* 主要内容：树形列表 */}
      <ChunkTreeView chunkId={chunkId} />

      {/* 可选：切换到图视图 */}
      <button onClick={() => showGraphView(chunkId)}>
        View as Graph
      </button>
    </div>
  );
}
```

---

## 数据查询 API 设计

### 获取单个 Chunk 的树

```typescript
// GET /api/chunks/:id/tree
{
  "chunk": {
    "id": "1",
    "names": ["main"],
    "size": 102400
  },
  "modules": [
    {
      "id": "m1",
      "name": "./src/index.js",
      "size": 1024,
      "module_kind": "Normal",
      "children": null
    },
    {
      "id": "m5",
      "name": "./src/utils.js",
      "size": 512,
      "module_kind": "Concatenated",
      "children": [
        { "id": "m5-1", "name": "./src/utils/a.js", "size": 200 },
        { "id": "m5-2", "name": "./src/utils/b.js", "size": 312 }
      ]
    }
  ]
}
```

### 获取所有 Chunks（不包含 Module 详情）

```typescript
// GET /api/chunks
[
  { "id": "1", "names": ["main"], "size": 102400, "moduleCount": 50 },
  { "id": "2", "names": ["vendor"], "size": 204800, "moduleCount": 120 }
]
```

---

## 总结

### 你的方案是正确的 ✅

在你描述的交互场景下：

```
选定 Chunk → 展示 Modules → 展开 Concatenated
```

这**确实是树形结构**，原因是：

1. **限定了上下文**（单个 Chunk）
2. **每个 Module 在视图中只出现一次**
3. **Concatenated Module 的展开是天然的树**

### 最佳实践建议

1. **默认使用树视图**（你的方案）
   - 简单、快速、符合直觉

2. **为共享模块添加提示**
   - 显示 "Shared" 标记
   - 工具提示显示其他 Chunk

3. **提供可选的图视图**
   - 用于高级分析场景
   - 可以跳转到图视图查看完整关系

4. **性能优化**
   - 虚拟滚动（Module 列表可能很长）
   - 懒加载（按需展开 Concatenated Module）

你的设计思路非常好！通过"限定上下文"将复杂的图结构简化为树，这是优秀 UI 设计的典范 🎯
