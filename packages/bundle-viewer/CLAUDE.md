# Bundle Viewer

> **🚨 重要提醒**：修改任何代码前，请先查看 [Checklist](#Checklist)
>
> **📖 快速导航**：[技术栈](#技术栈) | [数据模型](#数据模型架构) | [文件结构](#核心文件结构) | [Checklist](#Checklist) | [颜色系统](#-颜色系统-) | [优化评分](#优化评分系统)

## 项目概述

**构建数据可视化分析工具**，帮助开发者和管理者分析构建体积、优化配置、识别性能问题。

### 核心价值

- 📊 **科学评分**：基于 Core Web Vitals 影响的加权评分系统
- 🎯 **智能建议**：区分可自动优化和需人工评估的问题
- 🇨🇳 **中文优先**：所有界面文案统一使用中文
- 👥 **双重受众**：同时服务开发者（技术细节）和管理者（宏观概览）

---

## 技术栈

- **框架**: React 19.2 + TypeScript 5.9
- **构建**: Vite 7.3 + vite-plugin-singlefile (单文件 HTML)
- **样式**: Tailwind CSS v4.2 (oklch 色值)
- **UI 组件**: shadcn/ui (Radix UI)
- **包管理**: Bun (bun.lock)

---

## 数据模型架构

采用**标准化数据模型 + 适配层**架构：

```
JSON 文件上传
    ↓
window.__bundle_viewer_data__ (全局变量)
    ↓
useBundleData Hook (数据加载)
    ↓
adaptToStandardBundleData (data-adapter.ts)
    ↓
StandardBundleData (标准化模型)
    ↓
├─ use-optimization-data.ts (优化计算)
├─ use-chunk-relations.ts (依赖分析)
└─ 各页面组件
```

**核心接口**：
```typescript
StandardBundleData {
  modules: Module[]              // 模块列表
  chunks: Chunk[]                // 代码块列表
  packages: Package[]            // npm 包列表
  assets: Asset[]                // 静态资源
  moduleMap: Map<number, Module> // 快速查找
  chunkMap: Map<string, Chunk>
  packageMap: Map<string, Package[]>
  summary: BuildSummary          // 统计汇总
}

Module {
  id: number
  name: string                   // 统一路径（核心字段）
  size: number
  type: ModuleType               // 'javascript'|'css'|'json'|'asset'
  isNodeModule: boolean
  chunkIds: string[]
  packageName?: string
  ext?: { ... }                  // 工具特定字段
}

Chunk {
  id: string
  names: string[]
  size: number
  type: ChunkType                // 'entry'|'initial'|'async'|'runtime'
  moduleIds: number[]
  parentIds: string[]
  childIds: string[]
}
```

**设计原则**：
- `Module.name` / `Chunk.type` - 类型枚举（替代多个布尔字段）
- `ext` - 隔离调试字段，保持核心模型简洁
- `Map` 数据结构 - O(1) 查找性能

详见：[src/types/bundle-data-standard.d.ts](src/types/bundle-data-standard.d.ts)

---

## 核心文件结构

```
src/
├── app.tsx                          # 主应用入口（useTransition 优化）
├── main.tsx                         # React 渲染入口
├── index.css                        # oklch 色值系统 + 10色可视化色板
├── types/
│   ├── bundle-data.d.ts             # 旧格式（兼容）
│   └── bundle-data-standard.d.ts    # 标准模型 ⭐
├── components/
│   ├── optimization-suggestions.tsx # 🎯 优化建议（核心页面）
│   ├── summary.tsx                  # 📊 概览仪表盘
│   ├── package-analysis.tsx         # 📦 依赖包分析
│   ├── duplication-analysis.tsx     # 🔄 模块重复度
│   ├── chunk-viewer.tsx             # 🔷 代码块可视化
│   ├── module-viewer.tsx            # 📄 模块浏览器
│   ├── module-relations.tsx         # 🔗 模块依赖关系
│   ├── app-header.tsx               # 固定顶部导航
│   ├── file-upload-area.tsx         # 文件上传拖拽区
│   ├── package-card.tsx             # 包卡片组件
│   ├── chunks/                      # Chunk 子组件 (7个)
│   │   ├── chunk-analysis.tsx
│   │   ├── chunk-tree-view.tsx
│   │   ├── chunk-tree-node.tsx
│   │   ├── chunk-card.tsx
│   │   ├── chunk-stats-cards.tsx
│   │   ├── chunk-dependency-graph.tsx
│   │   └── circular-dependency-warning.tsx
│   ├── modules/                     # Module 子组件 (4个)
│   │   ├── module-explorer.tsx
│   │   ├── module-tree-view.tsx
│   │   ├── module-card.tsx
│   │   └── module-chunk-matrix.tsx
│   ├── optimization/                # 优化建议子组件 (9个)
│   │   ├── optimization-score-card.tsx
│   │   ├── circular-progress.tsx   # 圆形进度条
│   │   ├── potential-savings-card.tsx
│   │   ├── benefits-card.tsx
│   │   ├── chunk-quality-card.tsx
│   │   ├── comparison-cards.tsx    # 当前 vs 优化后
│   │   ├── quick-actions-panel.tsx
│   │   ├── optimization-actions.tsx
│   │   └── tables/                  # 优化数据表格 (3个)
│   │       ├── duplicate-packages-table.tsx
│   │       ├── large-packages-table.tsx
│   │       └── large-modules-table.tsx
│   └── ui/                          # shadcn/ui 组件 ⭐ (11个)
│       ├── badge.tsx, button.tsx, card.tsx, tabs.tsx
│       ├── table.tsx, input.tsx, tooltip.tsx
│       ├── scroll-area.tsx, collapsible.tsx
│       ├── copy-button.tsx (自定义)
│       └── percentage-bar.tsx (自定义可视化进度条)
├── hooks/                           # React Hooks (7个)
│   ├── index.ts                     # 统一导出
│   ├── use-bundle-data.ts           # Bundle 数据加载
│   ├── use-file-upload.ts           # 文件上传处理
│   ├── use-optimization-data.ts     # 优化计算（useMemo 集合）
│   ├── use-navigation.ts            # 导航状态（URL 同步）
│   ├── use-theme.ts                 # 深色/浅色主题
│   └── use-chunk-relations.ts       # Chunk 依赖分析
└── lib/
    ├── data-adapter.ts              # 数据适配器 ⭐
    ├── package-analyzer.ts          # 依赖包分析器
    ├── duplication-analyzer.ts      # 重复度检测
    ├── tree-builder.ts              # 树形结构构建
    ├── chunk-composition-analyzer.ts # Chunk 组成分析
    ├── optimization-utils.ts        # Semver 去重逻辑
    ├── navigation.ts                # URL 状态管理
    ├── constants.ts                 # 阈值/权重/UI限制
    ├── utils.ts                     # cn() 类名合并 ⭐
    └── utils/                       # 工具函数子模块 (5个)
        ├── index.ts                 # 统一导出
        ├── format.ts                # 字节/时间格式化
        ├── file-utils.ts            # 文件类型判断
        ├── path-utils.ts            # 路径美化
        └── export-utils.ts          # CSV/Markdown 导出
```

---

## 代码风格规范

### 1. UI 组件和样式工具 ⭐⭐⭐

#### 使用 cn() 处理类名合并

**必须使用** `cn()` 合并 Tailwind 类名：

```tsx
import { cn } from '@/lib/utils'

// ✅ 正确
<Button className={cn('w-full', isActive && 'bg-primary', className)} />
<div className={cn('p-4', disabled && 'opacity-50 cursor-not-allowed')} />

// ❌ 错误 - 手动拼接
<Button className={`w-full ${isActive ? 'bg-primary' : ''}`} />
```

**cn() 优势**：自动去重冲突类、智能处理条件、支持多种格式

#### 优先使用 shadcn/ui 组件

**必须优先使用** `src/components/ui/` 下的组件：

```tsx
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Table, TableBody, TableCell, TableHead } from '@/components/ui/table'

// ✅ 正确
<Button variant="default" size="sm">操作</Button>
<Badge variant="destructive">错误</Badge>

// ❌ 错误 - 自定义实现
<button className="px-4 py-2 rounded bg-primary">操作</button>
```

**可用组件**：button, badge, card, table, tabs, input, tooltip, scroll-area, collapsible, copy-button, percentage-bar (自定义)

### 2. React 组件模式

```tsx
import { memo, useMemo, useTransition } from 'react'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { formatBytes } from '@/lib/utils/format'

// ✅ 工具函数放组件外部或导入
function calculateScore(data: Data): number { ... }

// ✅ 子组件用 memo 避免重渲染
const CircularProgress = memo(({ score, className }: Props) => (
  <div className={cn('relative w-32 h-32', className)}>
    <svg>...</svg>
  </div>
))
CircularProgress.displayName = 'CircularProgress'

// ✅ 复杂计算用 useMemo（依赖数组精确）
const duplicatePackages = useMemo(() => {
  return packages.filter(p => p.versions.length > 1)
}, [packages])

// ✅ Tab 切换用 useTransition（非阻塞）
const [isPending, startTransition] = useTransition()
const handleTabChange = (tab: string) => {
  startTransition(() => setActiveTab(tab))
}

// ✅ 类名合并用 cn()
<Card className={cn('p-6', isPending && 'opacity-50', className)} />
```

### 3. 数据格式化

- **体积**: `formatBytes()` → "1.23 MB"
- **时间**: `formatTime()` → "1.23s" 或 "123.45ms"
- **数量**: `formatNumber()` → "1,234"

---

## 设计规范

### 🇨🇳 中文化原则

**所有面向用户的文案必须使用中文**：
- 页面标题、按钮文本（"展开/收起" 而非 "Expand"）
- 表头（"名称/体积/类型" 而非 "Name/Size"）
- 提示、错误信息、无数据状态

**例外**：代码术语（chunk、module）、技术名词（Gzip、dedupe、semver）

### 🎨 颜色系统 ⭐⭐⭐

#### 核心原则：oklch 色值 + 透明度分层

**主题色（oklch 色值系统）**：
```css
/* index.css 定义 */
--primary: oklch(0.6 0.1 185)        /* 钴蓝 - 专业、信任 */
--secondary: oklch(0.967 0.001 286)  /* 浅紫 - 次要 */
```

**进度条层级（透明度区分优先级）**：
```tsx
bg-primary opacity-80  // 高优先级
bg-primary opacity-60  // 中优先级
bg-primary opacity-40  // 低优先级
```

**背景装饰**：
```tsx
bg-primary/5   // 极浅（卡片光晕）
bg-primary/10  // 浅背景
bg-primary/15  // 中等背景
bg-primary/30  // 边框强调
```

**数据可视化色板（10色，色相环均匀分布）**：
```css
/* 用于图表、热力图等数据可视化 */
viz-1: oklch(0.6 0.1 200)   /* 钴蓝 - 最常用 */
viz-2: oklch(0.7 0.12 150)  /* 碧绿 */
viz-3: oklch(0.65 0.15 270) /* 蓝紫 */
viz-4: oklch(0.65 0.1 180)  /* 青蓝 */
viz-5: oklch(0.7 0.15 130)  /* 翠绿 */
viz-6: oklch(0.6 0.15 240)  /* 靛蓝 */
viz-7: oklch(0.68 0.12 160) /* 青绿 */
viz-8: oklch(0.62 0.18 290) /* 紫色 */
viz-9: oklch(0.72 0.1 170)  /* 浅青 */
viz-10: oklch(0.58 0.12 220) /* 宝石蓝 */
```

**中性色**：
- `text-foreground` - 主要文本
- `text-muted-foreground` - 次要文本
- `bg-background` - 页面背景
- `bg-muted` - 卡片背景
- `border-border` - 边框

**语义化颜色例外**：
- ✅ **红色**: `text-red-600`, `text-destructive` - 错误、严重警告
- ✅ **绿色**: `text-green-600`, `bg-green-600` - 成功状态
- ⛔ **禁用**: 装饰性彩虹色（蓝/紫/黄/橙/粉/青）

#### 代码示例

```tsx
// ✅ 正确 - 统一主题色 + oklch 色值
<div className="h-2 bg-muted rounded-full">
  <div className="h-full bg-primary opacity-80" style={{ width: `${percent}%` }} />
</div>

<Card className="border-primary/30 relative">
  {/* 背景光晕装饰 */}
  <div className="absolute top-0 right-0 w-32 h-32 bg-primary/5 rounded-full blur-3xl" />
</Card>

// ✅ 数据可视化使用 10 色色板
<div className="h-4 bg-viz-1" style={{ width: `${percent}%` }} />
<div className="h-4 bg-viz-2" style={{ width: `${percent}%` }} />

// ❌ 错误 - 多色渐变和任意颜色
<div className="h-full bg-linear-to-r from-yellow-500 to-red-600" />
<Card className="border-blue-500/50">  {/* 不使用装饰性蓝色 */}
```

### 📦 徽章系统

| 变体          | 用途                           | 示例               |
| ------------- | ------------------------------ | ------------------ |
| `destructive` | 严重问题                       | 重复依赖、循环依赖 |
| `outline`     | 中性信息、数量统计             | "123 个模块"       |
| `secondary`   | 次要分类                       | 模块类型、包类型   |
| `default`     | 一般信息、成功状态             | "优化良好"         |
| 绿色          | `className="bg-green-600"`明确成功 | "节省 2.3 MB"      |

### 🎯 交互模式

- 使用 Collapsible 实现渐进式披露
- 悬停显示 tooltip (title 属性)
- 分页加载而非无限滚动
- 入场动画：`animate-fade-in`

---

## 优化评分系统

### 加权评分 (总分 100)

```typescript
const WEIGHTS = {
  duplicateDeps: { max: 25, critical: 5, high: 3 },  // 重复依赖
  entryChunkSize: { max: 30, perChunk: 15 },         // Entry chunk (影响 FCP)
  entryPercentage: { max: 20, threshold: 50 },       // 代码分割质量
  asyncChunks: { max: 15, threshold: 3 },            // 懒加载利用率
  largePackages: { max: 10, threshold: 1024*1024 },  // 超大包 (>1MB)
}
```

### 性能阈值

- **Entry chunk**: 244KB (行业标准，直接影响 FCP)
- **大型包**: 100KB 警告 | 1MB 需优化
- **Entry 占比**: 50% (代码分割质量线)

### Semver 去重识别

- 相同主版本 (1.2.0 vs 1.3.0) → "可 dedupe"
- 不同主版本 (1.x vs 2.x) → "需评估" (breaking changes)

---

## 开发规范 Checklist

### ⭐ 最高优先级

**颜色使用**：
- [ ] 进度条使用 `bg-primary` + 透明度
- [ ] 主文本 `text-foreground`，次文本 `text-muted-foreground`
- [ ] 仅错误用红色，仅成功用绿色
- [ ] 禁用蓝/紫/黄/橙/粉/青

**UI 组件**：
- [ ] 使用 `cn()` 合并类名
- [ ] 优先使用 shadcn/ui 组件
- [ ] Badge 使用内置变体

### 中文化

- [ ] 所有标题、按钮、表头使用中文
- [ ] 错误提示、tooltip、无数据状态
- [ ] 搜索框 placeholder

### 性能和体验

- [ ] 大列表使用分页或虚拟滚动
- [ ] 复杂计算用 `useMemo`
- [ ] 子组件用 `memo` 优化
- [ ] 长内容用 `truncate` + `title`
- [ ] 页面添加 `animate-fade-in`

---

## 7 个功能页面

| 选项卡 | 文件 | 核心功能 |
|--------|------|---------|
| 🎯 优化建议 | `optimization-suggestions.tsx` | 评分系统、潜在收益、对比卡片、建议表格 |
| 📊 概览 | `summary.tsx` | 统计卡片、Top 10 列表、资源分析 |
| 📦 依赖包 | `package-analysis.tsx` | 包列表、多版本识别、代码分布饼图 |
| 🔄 重复度 | `duplication-analysis.tsx` | 模块重复检测、严重度分级、热力图 |
| 🔷 代码块 | `chunk-viewer.tsx` | Chunk 树、组成分析、依赖图、循环依赖警告 |
| 📄 模块 | `module-viewer.tsx` | 模块树浏览、搜索、Chunk-Module 矩阵 |
| 🔗 依赖关系 | `module-relations.tsx` | 模块间依赖关系可视化 |

**导航特性**：
- Tab 切换使用 `useTransition` 优化体验（非阻塞）
- URL 参数同步：`?tab=optimization&moduleName=...`
- 支持深链接和页面状态持久化

---

## 常见问题

**Q: 为什么单文件 HTML？**
A: 方便分享部署，无需服务器，浏览器直接打开。

**Q: 评分权重依据？**
A: 基于对 Core Web Vitals (FCP/LCP) 影响：Entry chunk 大小 (30%) > 重复依赖 (25%) > 代码分割质量 (20%)

**Q: "可 dedupe" vs "需评估"？**
A: 相同主版本可直接 `npm dedupe`；不同主版本需人工确认 breaking changes

---

## 性能优化策略

### 1. 数据结构优化
- **标准化模型**: 避免重复格式转换
- **Map 快速查找**: `moduleMap`, `chunkMap`, `packageMap`
- **预计算**: 适配器阶段完成类型推断和包名提取

### 2. React 渲染优化
- **useTransition**: Tab 切换非阻塞（优先级调度）
- **useMemo 集合**: `use-optimization-data.ts` 集中管理所有计算
- **组件 memo**: 防止不必要的重渲染
- **虚拟滚动**: 大列表性能保证

### 3. UI 体验优化
- **固定表头**: 表格可滚动但表头固定
- **渐进式披露**: Collapsible 隐藏次要信息
- **入场动画**: `animate-fade-in` 减少突兀感
- **Loading 状态**: 优雅的加载指示器

### 4. 构建优化
- **单文件输出**: 无需服务器，分享便捷
- **内联资源**: CSS/JS 内联，减少请求
- **测试版本**: 嵌入示例数据，无需上传

---

## 构建命令

```bash
bun run dev    # 开发服务器 (http://localhost:5173)
bun run build  # 同时生成生产版和测试版
```

**构建产物**：
- `dist/index.html` - 生产版（需上传 JSON）
- `dist/index.test.html` - 测试版（内置 data/bmas_db.json）

**构建流程**：`tsc -b` → `vite build` → `vite-plugin-singlefile` → 单文件 HTML

---

## 设计系统速查

### 常用颜色

| 用途     | 类名                              | oklch 值 | 说明 |
| -------- | --------------------------------- | -------- | ---- |
| 主色调   | `bg-primary`, `text-primary`      | oklch(0.6 0.1 185) | 钴蓝 |
| 次要色   | `bg-secondary`, `text-secondary`  | oklch(0.967 0.001 286) | 浅紫 |
| 进度条   | `bg-primary opacity-80/60/40`     | - | 透明度分层 |
| 浅背景   | `bg-primary/5` ~ `/30`            | - | 卡片装饰/边框 |
| 主文本   | `text-foreground`                 | - | 正文 |
| 次文本   | `text-muted-foreground`           | - | 说明 |
| 成功     | `text-green-600`, `bg-green-600`  | - | 仅成功状态 |
| 错误     | `text-red-600`, `text-destructive` | - | 仅错误/警告 |
| 可视化   | `bg-viz-1` ~ `bg-viz-10`          | oklch(...) | 图表/热力图 |

### Badge 变体

| 变体 | 类名 | 用途 |
| ---- | ---- | ---- |
| destructive | `<Badge variant="destructive">` | 严重问题、错误 |
| outline | `<Badge variant="outline">` | 中性信息、数量 |
| secondary | `<Badge variant="secondary">` | 次要分类 |
| default | `<Badge variant="default">` | 一般信息、成功 |

**统一样式常量**（来自 `constants.ts`）：
```tsx
import { BADGE_STYLES } from '@/lib/constants'

<Badge className={BADGE_STYLES.COMPACT}>通用</Badge>
<Badge className={BADGE_STYLES.SIZE}>1.23 MB</Badge>
<Badge className={BADGE_STYLES.NUMERIC}>v1.2.3</Badge>
```

### 响应式布局

```tsx
<div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">        // 自适应网格
<div className="w-full max-w-7xl mx-auto px-8">                   // 固定宽度容器
<div className="grid grid-cols-2 md:grid-cols-4 gap-4">          // 统计卡片网格
<div className="flex flex-col lg:flex-row gap-6">                // 响应式 Flex
```
---

## Checklist

- [ ] 类名合并用 `cn()`，禁止字符串拼接
- [ ] 优先用 shadcn/ui 组件：`Button` `Badge` `Card` `Table` 等
- [ ] 进度条用 `bg-primary` + 透明度，禁止彩虹色
- [ ] 主文本 `text-foreground`，次文本 `text-muted-foreground`
- [ ] 仅错误用红色，仅成功用绿色，禁用蓝/紫/黄/橙等
- [ ] Badge 用内置变体：`destructive` `outline` `secondary` `default`
- [ ] 所有用户可见文案使用中文（按钮/表头/提示/placeholder）
- [ ] TypeScript 类型完整
- [ ] 长文本用 `truncate` + `title`

**可用组件**：button, badge, card, table, tabs, input, tooltip, scroll-area, collapsible, copy-button, percentage-bar
