import type { StandardBundleData } from '@/types/bundle-data-standard'

/** Chunk 组成类别 */
export type ChunkCompositionCategory =
  | 'pure-lib' // 纯第三方库 (>95% lib)
  | 'lib-dominant' // lib主导 (70-95% lib)
  | 'balanced' // 均衡 (30-70% lib)
  | 'source-dominant' // 源码主导 (5-30% lib)
  | 'pure-source' // 纯源码 (<5% lib)

/** Chunk 组成分析结果 */
export interface ChunkComposition {
  chunkId: string
  chunkNames: string[]
  size: number
  type: 'entry' | 'initial' | 'async' | 'runtime'

  // 组成分析
  libSize: number // 第三方库体积
  sourceSize: number // 源码体积
  libPercentage: number // lib占比 (0-100)
  sourcePercentage: number // 源码占比 (0-100)
  category: ChunkCompositionCategory // 分类

  // 模块详情
  libModules: number // 第三方模块数
  sourceModules: number // 源码模块数
  totalModules: number
}

/** 分类统计 */
export interface CompositionDistribution {
  category: ChunkCompositionCategory
  count: number
  totalSize: number
  chunks: ChunkComposition[]
}

/** 完整分析结果 */
export interface ChunkCompositionAnalysisResult {
  chunks: ChunkComposition[]
  distribution: CompositionDistribution[]
  stats: {
    totalChunks: number
    totalSize: number
    totalLibSize: number
    totalSourceSize: number
    avgLibPercentage: number
  }
}

/**
 * 根据 lib 占比确定分类
 */
function categorizeByLibPercentage(libPercentage: number): ChunkCompositionCategory {
  if (libPercentage > 95) return 'pure-lib'
  if (libPercentage >= 70) return 'lib-dominant'
  if (libPercentage >= 30) return 'balanced'
  if (libPercentage >= 5) return 'source-dominant'
  return 'pure-source'
}

/**
 * 获取分类的中文名称
 */
export function getCategoryName(category: ChunkCompositionCategory): string {
  const names: Record<ChunkCompositionCategory, string> = {
    'pure-lib': '纯第三方库',
    'lib-dominant': '第三方库主导',
    balanced: '均衡混合',
    'source-dominant': '源码主导',
    'pure-source': '纯源码',
  }
  return names[category]
}

/**
 * 获取分类的颜色类
 */
export function getCategoryColor(category: ChunkCompositionCategory): string {
  const colors: Record<ChunkCompositionCategory, string> = {
    'pure-lib': 'text-purple-600 dark:text-purple-400',
    'lib-dominant': 'text-blue-600 dark:text-blue-400',
    balanced: 'text-green-600 dark:text-green-400',
    'source-dominant': 'text-orange-600 dark:text-orange-400',
    'pure-source': 'text-pink-600 dark:text-pink-400',
  }
  return colors[category]
}

/**
 * 获取分类的徽章变体
 */
export function getCategoryBadgeVariant(
  category: ChunkCompositionCategory,
): 'default' | 'secondary' | 'destructive' | 'outline' {
  const variants: Record<ChunkCompositionCategory, 'default' | 'secondary' | 'destructive' | 'outline'> = {
    'pure-lib': 'default',
    'lib-dominant': 'secondary',
    balanced: 'outline',
    'source-dominant': 'secondary',
    'pure-source': 'default',
  }
  return variants[category]
}

/**
 * 分析 Chunk 的 lib/source 组成
 */
export function analyzeChunkComposition(data: StandardBundleData): ChunkCompositionAnalysisResult {
  // 直接使用标准结构的 moduleMap（按 ID 索引）
  const moduleMap = data.moduleMap

  const chunks: ChunkComposition[] = []

  // 分析每个 chunk
  for (const chunk of data.chunks) {
    let libSize = 0
    let sourceSize = 0
    let libModules = 0
    let sourceModules = 0

    // 遍历 chunk 中的所有模块
    for (const moduleId of chunk.moduleIds) {
      const module = moduleMap.get(moduleId)
      if (!module) continue

      if (module.isNodeModule) {
        libSize += module.size
        libModules++
      } else {
        sourceSize += module.size
        sourceModules++
      }
    }

    const totalSize = libSize + sourceSize
    const libPercentage = totalSize > 0 ? (libSize / totalSize) * 100 : 0
    const sourcePercentage = 100 - libPercentage
    const category = categorizeByLibPercentage(libPercentage)

    chunks.push({
      chunkId: chunk.id,
      chunkNames: chunk.names,
      size: chunk.size,
      type: chunk.type,
      libSize,
      sourceSize,
      libPercentage,
      sourcePercentage,
      category,
      libModules,
      sourceModules,
      totalModules: chunk.moduleIds.length,
    })
  }

  // 按体积降序排序
  chunks.sort((a, b) => b.size - a.size)

  // 统计分布
  const distributionMap = new Map<ChunkCompositionCategory, CompositionDistribution>()
  const categories: ChunkCompositionCategory[] = [
    'pure-lib',
    'lib-dominant',
    'balanced',
    'source-dominant',
    'pure-source',
  ]

  for (const category of categories) {
    distributionMap.set(category, {
      category,
      count: 0,
      totalSize: 0,
      chunks: [],
    })
  }

  for (const chunk of chunks) {
    const dist = distributionMap.get(chunk.category)!
    dist.count++
    dist.totalSize += chunk.size
    dist.chunks.push(chunk)
  }

  const distribution = Array.from(distributionMap.values())

  // 计算总体统计
  const totalChunks = chunks.length
  const totalSize = chunks.reduce((sum, c) => sum + c.size, 0)
  const totalLibSize = chunks.reduce((sum, c) => sum + c.libSize, 0)
  const totalSourceSize = chunks.reduce((sum, c) => sum + c.sourceSize, 0)
  const avgLibPercentage = totalSize > 0 ? (totalLibSize / totalSize) * 100 : 0

  return {
    chunks,
    distribution,
    stats: {
      totalChunks,
      totalSize,
      totalLibSize,
      totalSourceSize,
      avgLibPercentage,
    },
  }
}
