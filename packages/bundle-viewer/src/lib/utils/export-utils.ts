/**
 * 数据导出工具
 */

import type { StandardBundleData } from '@/types/bundle-data-standard'
import { formatBytes } from './format'

/**
 * 导出模块数据为 CSV
 */
export function exportToCSV(data: StandardBundleData) {
  // 导出最大的 100 个模块
  const modules = data.modules.toSorted((a, b) => b.size - a.size).slice(0, 100)

  const csvRows = [
    ['模块名称', '体积(Bytes)', '类型', '是否为node_modules', '代码块数量'].join(','),
    ...modules.map((m) =>
      [`"${m.name.replace(/"/g, '""')}"`, m.size, m.type, m.isNodeModule ? '是' : '否', m.chunkIds.length].join(','),
    ),
  ]

  const csvContent = csvRows.join('\n')
  const blob = new Blob(['\ufeff' + csvContent], { type: 'text/csv;charset=utf-8;' })
  downloadFile(blob, `模块分析-${getDateString()}.csv`)
}

/**
 * 导出分析报告为 Markdown
 */
export function exportToMarkdown(data: StandardBundleData) {
  const date = new Date().toLocaleDateString()

  let markdown = `# 构建分析报告\n\n`
  markdown += `**生成时间：** ${date}\n\n`

  // 概览部分
  markdown += buildOverviewSection(data)

  // 依赖包分析
  markdown += buildPackageSection(data)

  // 代码块分析
  markdown += buildChunkSection(data)

  // 优化建议
  markdown += buildOptimizationSection(data)

  markdown += `---\n\n*由 Bundle Viewer 生成*\n`

  const blob = new Blob([markdown], { type: 'text/markdown' })
  downloadFile(blob, `构建分析报告-${date.replace(/\//g, '-')}.md`)
}

/**
 * 构建概览部分
 */
function buildOverviewSection(data: StandardBundleData): string {
  let section = `## 概览\n\n`
  section += `- **总体积：** ${formatBytes(data.summary.totalSize)}\n`
  if (data.summary.totalGzipSize > 0) {
    section += `- **Gzip 压缩后：** ${formatBytes(data.summary.totalGzipSize)}\n`
  }
  section += `- **模块总数：** ${data.summary.totalModules.toLocaleString()}\n`
  section += `- **代码块数量：** ${data.summary.totalChunks}\n`
  section += `- **资源文件数：** ${data.summary.totalAssets}\n\n`
  return section
}

/**
 * 构建依赖包部分
 */
function buildPackageSection(data: StandardBundleData): string {
  const packageGroups = new Map<string, Set<string>>()
  for (const pkg of data.packages) {
    if (!packageGroups.has(pkg.name)) {
      packageGroups.set(pkg.name, new Set())
    }
    packageGroups.get(pkg.name)!.add(pkg.version)
  }
  const multiVersion = Array.from(packageGroups.entries()).filter(([, versions]) => versions.size > 1)

  let section = `## 依赖包分析\n\n`
  section += `- **独立包数量：** ${packageGroups.size}\n`
  section += `- **包实例总数：** ${data.packages.length}\n`

  if (multiVersion.length > 0) {
    section += `- **⚠️ 多版本依赖：** ${multiVersion.length}\n\n`
    section += `### 重复依赖详情\n\n`
    section += `| 包名 | 版本 |\n`
    section += `|------|------|\n`
    multiVersion.forEach(([name, versions]) => {
      section += `| ${name} | ${Array.from(versions).join(', ')} |\n`
    })
    section += `\n`
  } else {
    section += `- ✅ 未检测到重复版本\n\n`
  }

  // Top 10 最大依赖包
  const topPackages = data.packages.toSorted((a, b) => b.size - a.size).slice(0, 10)
  section += `### 最大的 10 个依赖包\n\n`
  section += `| 包名 | 版本 | 体积 |\n`
  section += `|------|------|------|\n`
  topPackages.forEach((pkg) => {
    section += `| ${pkg.name} | ${pkg.version} | ${formatBytes(pkg.size)} |\n`
  })
  section += `\n`

  return section
}

/**
 * 构建代码块部分
 */
function buildChunkSection(data: StandardBundleData): string {
  const entryChunks = data.chunks.filter((c) => c.type === 'entry')
  const asyncChunks = data.chunks.filter((c) => c.type === 'async')
  const largeEntryChunks = entryChunks.filter((c) => c.size > 244 * 1024)

  let section = `## 代码块分析\n\n`
  section += `- **入口代码块：** ${entryChunks.length}\n`
  section += `- **异步代码块：** ${asyncChunks.length}\n`

  if (largeEntryChunks.length > 0) {
    section += `- **⚠️ 过大的入口代码块 (>244KB)：** ${largeEntryChunks.length}\n\n`
    section += `### 过大的入口代码块详情\n\n`
    section += `| 代码块 | 体积 |\n`
    section += `|--------|------|\n`
    largeEntryChunks.forEach((chunk) => {
      section += `| ${chunk.names.join(', ') || chunk.id} | ${formatBytes(chunk.size)} |\n`
    })
    section += `\n`
  }

  return section
}

/**
 * 构建优化建议部分
 */
function buildOptimizationSection(data: StandardBundleData): string {
  const packageGroups = new Map<string, Set<string>>()
  for (const pkg of data.packages) {
    if (!packageGroups.has(pkg.name)) {
      packageGroups.set(pkg.name, new Set())
    }
    packageGroups.get(pkg.name)!.add(pkg.version)
  }
  const multiVersion = Array.from(packageGroups.entries()).filter(([, versions]) => versions.size > 1)

  const entryChunks = data.chunks.filter((c) => c.type === 'entry')
  const asyncChunks = data.chunks.filter((c) => c.type === 'async')
  const largeEntryChunks = entryChunks.filter((c) => c.size > 244 * 1024)

  let section = `## 优化建议\n\n`

  if (multiVersion.length > 0) {
    section += `### 🔴 高优先级：解决重复依赖\n\n`
    section += `检测到 ${multiVersion.length} 个包存在多个版本，这会不必要地增加构建体积。\n\n`
    section += `**操作建议：** 运行 \`npm dedupe\` 或使用包管理器的 resolutions 功能统一版本。\n\n`
  }

  if (largeEntryChunks.length > 0) {
    section += `### 🔴 高优先级：优化入口代码块\n\n`
    section += `${largeEntryChunks.length} 个入口代码块超过 244KB，会影响首次内容绘制时间（FCP）。\n\n`
    section += `**操作建议：**\n`
    section += `- 使用代码分割提取第三方库\n`
    section += `- 对路由实施动态导入\n`
    section += `- 考虑使用 tree-shaking 移除未使用的代码\n\n`
  }

  if (asyncChunks.length < 3) {
    section += `### 🟡 中等优先级：增加懒加载\n\n`
    section += `仅检测到 ${asyncChunks.length} 个异步代码块。更多的懒加载可以改善初始加载时间。\n\n`
    section += `**操作建议：** 对路由和重型功能使用动态导入 (\`import()\`)。\n\n`
  }

  if (multiVersion.length === 0 && largeEntryChunks.length === 0 && asyncChunks.length >= 3) {
    section += `### ✅ 优化良好！\n\n`
    section += `您的构建配置展现了良好的优化实践。请继续监控，防止性能退化。\n\n`
  }

  return section
}

/**
 * 下载文件辅助函数
 */
function downloadFile(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

/**
 * 获取日期字符串
 */
function getDateString(): string {
  return new Date().toISOString().split('T')[0]
}
