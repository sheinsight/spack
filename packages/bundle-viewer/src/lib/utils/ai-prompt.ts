import type { StandardBundleData } from '@/types/bundle-data-standard'
import { formatBytes } from '@/lib/utils/format'
import { buildOptimizationMetrics } from '@/lib/optimization-metrics'

export function buildAIFixPrompt(data: StandardBundleData): string {
  const metrics = buildOptimizationMetrics(data)
  const totalPotentialSavings = metrics.duplicatePackages.reduce((sum, p) => sum + p.potentialSavings, 0)

  const topDuplicates = metrics.duplicatePackages.slice(0, 8)
  const topLargePackages = metrics.largePackages.slice(0, 8)
  const topLargeModules = metrics.largeModules.slice(0, 8)
  const topLargeEntryChunks = metrics.chunkQuality.largeEntryChunks.slice(0, 5)

  const timestamp = data.timestamp ? new Date(data.timestamp).toLocaleString() : '未知'
  const buildTool = data.ext?.buildTool || '未知'

  const lines: string[] = []
  lines.push('你是资深前端构建优化与依赖治理专家。请基于以下“构建分析结果”生成修复方案。')
  lines.push('')
  lines.push('目标：给出可直接执行的优化方案（依赖治理、代码分割、懒加载、包体积控制），并标注优先级与预期收益。')
  lines.push('要求：')
  lines.push('- 优先给出 P0/P1 级别可落地措施')
  lines.push('- 指出建议修改的配置位置（webpack/vite/rollup 等通用思路即可）')
  lines.push('- 如信息不足，请提出最少且关键的问题')
  lines.push('')
  lines.push('【构建概览】')
  lines.push(`- 生成时间: ${timestamp}`)
  lines.push(`- 构建工具: ${buildTool}`)
  lines.push(`- 总体积: ${formatBytes(data.summary.totalSize)}`)
  if (data.summary.totalGzipSize > 0) {
    lines.push(`- Gzip: ${formatBytes(data.summary.totalGzipSize)}`)
  }
  lines.push(`- 模块总数: ${data.summary.totalModules}`)
  lines.push(`- 代码块数: ${data.summary.totalChunks}`)
  lines.push(`- 资源数: ${data.summary.totalAssets}`)
  lines.push('')
  lines.push('【代码分割现状】')
  lines.push(`- 入口体积: ${formatBytes(metrics.chunkQuality.entrySize)} (${metrics.chunkQuality.entryPercentage.toFixed(1)}%)`)
  lines.push(`- 异步体积: ${formatBytes(metrics.chunkQuality.asyncSize)} (${metrics.chunkQuality.asyncPercentage.toFixed(1)}%)`)
  lines.push(`- 异步代码块数: ${metrics.chunkQuality.asyncChunkCount}`)
  if (topLargeEntryChunks.length > 0) {
    lines.push(`- 超大入口块(>244KB): ${topLargeEntryChunks.length} 个`)
    topLargeEntryChunks.forEach((chunk) => {
      lines.push(`  - ${chunk.names.join(', ') || chunk.id}: ${formatBytes(chunk.size)}`)
    })
  } else {
    lines.push('- 超大入口块(>244KB): 无')
  }
  lines.push('')
  lines.push('【重复依赖】')
  lines.push(`- 多版本包数量: ${metrics.packageStats.multiVersion}`)
  lines.push(`- 可节省体积(估算): ${formatBytes(totalPotentialSavings)}`)
  if (topDuplicates.length > 0) {
    lines.push('- Top 重复依赖(按潜在节省排序):')
    topDuplicates.forEach((pkg) => {
      const versions = pkg.versions.map((v) => `${v.version} (${formatBytes(v.size)})`).join(', ')
      lines.push(`  - ${pkg.name}: ${versions} | 可省 ${formatBytes(pkg.potentialSavings)}`)
    })
  } else {
    lines.push('- Top 重复依赖: 无')
  }
  lines.push('')
  lines.push('【大型依赖包】')
  if (topLargePackages.length > 0) {
    topLargePackages.forEach((pkg) => {
      lines.push(`  - ${pkg.name}@${pkg.version}: ${formatBytes(pkg.size)} (${pkg.percentage.toFixed(1)}%)`)
    })
  } else {
    lines.push('  - 无明显超大依赖包')
  }
  lines.push('')
  lines.push('【大型模块】')
  if (topLargeModules.length > 0) {
    topLargeModules.forEach((mod) => {
      lines.push(`  - ${mod.displayName}: ${formatBytes(mod.size)}`)
    })
  } else {
    lines.push('  - 无明显超大模块')
  }
  lines.push('')
  lines.push('【请输出】')
  lines.push('1) 问题清单（P0/P1/P2 分类）')
  lines.push('2) 修复建议（含可执行操作、配置要点、预期收益）')
  lines.push('3) 验证步骤（构建前后如何对比）')
  lines.push('4) 需要我补充的最少关键信息')
  lines.push('')
  lines.push('注意：仅使用上述分析数据，不要猜测项目细节。')

  return lines.join('\n')
}
