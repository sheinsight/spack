import type { StandardBundleData, Chunk } from '@/types/bundle-data-standard'

/** 模块重复度统计 */
export interface ModuleDuplicationStat {
  // 模块基本信息
  moduleName: string
  moduleSize: number
  moduleType: 'project' | 'third-party' | 'builtin' | 'unknown'

  // 重复加载统计
  chunkCount: number // 出现在几个chunk中
  chunks: Array<{
    // 所在的chunks详情
    id: string
    names: string[]
    size: number
    type: 'entry' | 'initial' | 'async' | 'runtime'
  }>

  // 影响分析
  wastedSize: number // 浪费的体积 = size × (chunkCount - 1)
  impactScore: number // 影响评分（0-100）

  // 依赖信息
  concatenatedModules?: Array<{
    // 被合并的子模块
    id: number
    name: string
    size: number
  }>
}

/** 重复度分析结果 */
export interface DuplicationAnalysisResult {
  // 总体统计
  totalModules: number
  duplicatedModules: number // chunkCount > 1
  uniqueModules: number

  // 体积统计
  totalWastedSize: number
  totalModulesSize: number
  wastedPercentage: number

  // 重复度分布
  duplicationDistribution: {
    count2: number // 2x重复
    count3: number // 3x重复
    count4: number // 4x重复
    count5Plus: number // 5+x重复
  }

  // 详细数据
  modules: ModuleDuplicationStat[] // 按impactScore降序
  topDuplicated: ModuleDuplicationStat[] // Top 20
}

/**
 * 分析模块重复加载情况
 *
 * 算法流程：
 * 1. 遍历所有模块，统计每个模块出现在几个chunk中
 * 2. 计算浪费体积 = 模块体积 × (出现次数 - 1)
 * 3. 计算影响评分 = 归一化(chunkCount) × 0.4 + 归一化(wastedSize) × 0.6
 * 4. 按影响评分降序排序
 * 5. 统计重复度分布
 */
export function analyzeModuleDuplication(data: StandardBundleData): DuplicationAnalysisResult {
  // 创建 chunk 映射表，加速查询
  const chunkMap = new Map<string, Chunk>()
  for (const chunk of data.chunks) {
    chunkMap.set(chunk.id, chunk)
  }

  const moduleStats: ModuleDuplicationStat[] = []

  // 1. 遍历所有模块，统计重复情况
  for (const module of data.modules) {
    const chunkCount = module.chunkIds.length
    const moduleName = module.name

    // 获取chunk详情
    const chunks = module.chunkIds
      .map((chunkId) => {
        const chunk = chunkMap.get(chunkId)
        if (!chunk) return null
        return {
          id: chunk.id,
          names: chunk.names,
          size: chunk.size,
          type: chunk.type,
        }
      })
      .filter(Boolean) as ModuleDuplicationStat['chunks']

    // 计算浪费体积（只有重复的才算浪费）
    const wastedSize = chunkCount > 1 ? module.size * (chunkCount - 1) : 0

    // 处理 concatenatedModules
    const concatenatedModules = module.ext?.concatenatedModules?.map((cm) => ({
      id: cm.id,
      name: cm.name,
      size: cm.size,
    }))

    // 判断模块类型
    const moduleType: ModuleDuplicationStat['moduleType'] = module.isNodeModule ? 'third-party' : 'project'

    moduleStats.push({
      moduleName,
      moduleSize: module.size,
      moduleType,
      chunkCount,
      chunks,
      wastedSize,
      impactScore: 0, // 后续计算
      concatenatedModules,
    })
  }

  // 2. 计算影响评分（归一化）
  const maxChunkCount = Math.max(...moduleStats.map((s) => s.chunkCount), 1)
  const maxWastedSize = Math.max(...moduleStats.map((s) => s.wastedSize), 1)

  for (const stat of moduleStats) {
    // 影响评分 = (重复次数权重 × 0.4) + (浪费体积权重 × 0.6)
    const countScore = stat.chunkCount / maxChunkCount
    const sizeScore = stat.wastedSize / maxWastedSize
    stat.impactScore = (countScore * 0.4 + sizeScore * 0.6) * 100
  }

  // 3. 按影响评分降序排序
  const sortedModules = moduleStats.toSorted((a, b) => b.impactScore - a.impactScore)

  // 4. 统计汇总
  const duplicatedModules = sortedModules.filter((s) => s.chunkCount > 1)
  const totalWastedSize = duplicatedModules.reduce((sum, s) => sum + s.wastedSize, 0)
  const totalModulesSize = moduleStats.reduce((sum, s) => sum + s.moduleSize, 0)

  // 5. 重复度分布
  const distribution = {
    count2: duplicatedModules.filter((s) => s.chunkCount === 2).length,
    count3: duplicatedModules.filter((s) => s.chunkCount === 3).length,
    count4: duplicatedModules.filter((s) => s.chunkCount === 4).length,
    count5Plus: duplicatedModules.filter((s) => s.chunkCount >= 5).length,
  }

  return {
    totalModules: moduleStats.length,
    duplicatedModules: duplicatedModules.length,
    uniqueModules: moduleStats.length - duplicatedModules.length,
    totalWastedSize,
    totalModulesSize,
    wastedPercentage: totalModulesSize > 0 ? (totalWastedSize / totalModulesSize) * 100 : 0,
    duplicationDistribution: distribution,
    modules: sortedModules,
    topDuplicated: duplicatedModules.slice(0, 20),
  }
}
