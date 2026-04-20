# Bundle Viewer

构建分析可视化工具 - 单文件 HTML 查看器

## 项目简介

一个用于分析构建产物的可视化工具，支持分析各类构建工具生成的统计数据。所有功能构建在单个 HTML 文件中，无需服务器即可使用。

## 使用方法

### 1. 构建查看器

```bash
bun run build
```

这将在 `dist/index.html` 生成单文件 HTML。

### 2. 注入构建统计数据

打开 `dist/index.html`，找到以下部分：

```html
<script>
  // Data injection placeholder
  var DATA_PLACEHOLDER = null
  window.__bundle_viewer_data__ = DATA_PLACEHOLDER
</script>
```

将 `DATA_PLACEHOLDER` 替换为你的构建统计数据（JSON 格式）：

```html
<script>
  // Data injection placeholder
  var DATA_PLACEHOLDER = {"modules":[...],"chunks":[...],"assets":[...]};
  window.__bundle_viewer_data__ = DATA_PLACEHOLDER;
</script>
```

### 3. 打开文件

双击 `dist/index.html` 或在浏览器中打开即可。支持 `file://` 协议。

## 数据格式要求

统计数据需要包含以下字段：

- `modules`: 模块列表
- `chunks`: 代码块列表
- `assets`: 资源文件列表
- `summary`: 总体统计（可选）
- `packages`: npm 包列表（可选）

## Development

```bash
# Install dependencies
bun install

# Start dev server
bun run dev

# Build for production
bun run build
```
