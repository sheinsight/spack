import type { StandardBundleData, Package } from '@/types/bundle-data-standard'

export interface PackageVersion {
  version: string
  size: number
  moduleCount: number
  packageJsonPath: string
}

export interface PackageGroup {
  name: string
  versions: PackageVersion[]
  totalSize: number
  totalModuleCount: number
  versionCount: number
}

export interface CodeDistribution {
  thirdPartySize: number
  projectSize: number
  thirdPartyModules: number
  projectModules: number
}

export interface ChunkDistribution {
  thirdPartySize: number
  projectSize: number
  thirdPartyChunks: number
  projectChunks: number
  mixedChunks: number
  totalChunks: number
}

/**
 * 按包名分组，识别多版本包
 */
export function groupPackagesByName(packages: Package[]): PackageGroup[] {
  const grouped = new Map<string, PackageGroup>()

  for (const pkg of packages) {
    if (!grouped.has(pkg.name)) {
      grouped.set(pkg.name, {
        name: pkg.name,
        versions: [],
        totalSize: 0,
        totalModuleCount: 0,
        versionCount: 0,
      })
    }

    const group = grouped.get(pkg.name)!
    group.versions.push({
      version: pkg.version,
      size: pkg.size,
      moduleCount: pkg.moduleCount,
      packageJsonPath: pkg.ext?.packageJsonPath || '',
    })
    group.totalSize += pkg.size
    group.totalModuleCount += pkg.moduleCount
    group.versionCount = group.versions.length
  }

  // 转换为数组并按版本数量降序排序，同时对每个包的版本排序
  return Array.from(grouped.values(), (group) => ({
    ...group,
    versions: group.versions.toSorted((a, b) => b.size - a.size),
  })).toSorted((a, b) => b.versionCount - a.versionCount)
}

/**
 * 分析第三方代码和项目代码的占比
 */
export function analyzeCodeDistribution(data: StandardBundleData): CodeDistribution {
  // 使用 reduce 一次遍历完成统计，避免可变状态累加
  return data.modules.reduce<CodeDistribution>(
    (result, module) => {
      if (module.isNodeModule) {
        result.thirdPartySize += module.size
        result.thirdPartyModules++
      } else {
        result.projectSize += module.size
        result.projectModules++
      }
      return result
    },
    {
      thirdPartySize: 0,
      projectSize: 0,
      thirdPartyModules: 0,
      projectModules: 0,
    },
  )
}

/**
 * 分析 chunks 中的第三方代码和项目代码占比
 *
 * ⚠️ 重要：此函数计算的是"去重后的实际体积"，不会重复计算共享模块。
 * 如果一个模块被多个 chunk 引用，其体积只会被计算一次。
 */
export function analyzeChunkDistribution(data: StandardBundleData): ChunkDistribution {
  // 直接使用标准结构的 moduleMap（按 ID 索引）
  const moduleMap = data.moduleMap

  const result: ChunkDistribution = {
    thirdPartySize: 0,
    projectSize: 0,
    thirdPartyChunks: 0,
    projectChunks: 0,
    mixedChunks: 0,
    totalChunks: data.chunks.length,
  }

  // 使用 Set 记录已计算过的模块，避免重复计数
  const countedModules = new Set<number>()

  // 遍历所有 chunks
  for (const chunk of data.chunks) {
    let chunkThirdPartyModules = 0
    let chunkProjectModules = 0

    // 统计这个 chunk 包含的所有 modules
    for (const moduleId of chunk.moduleIds) {
      const module = moduleMap.get(moduleId)
      if (!module) continue

      // 统计 chunk 级别的分类（用于判断 chunk 类型）
      if (module.isNodeModule) {
        chunkThirdPartyModules++
      } else {
        chunkProjectModules++
      }

      // 全局去重统计（避免重复计数）
      if (!countedModules.has(moduleId)) {
        countedModules.add(moduleId)

        if (module.isNodeModule) {
          result.thirdPartySize += module.size
        } else {
          result.projectSize += module.size
        }
      }
    }

    // 分类统计 chunk（基于该 chunk 的主要内容）
    if (chunkThirdPartyModules > 0 && chunkProjectModules > 0) {
      result.mixedChunks++
    } else if (chunkThirdPartyModules > 0) {
      result.thirdPartyChunks++
    } else if (chunkProjectModules > 0) {
      result.projectChunks++
    }
  }

  return result
}

/**
 * 过滤包组列表（支持搜索）
 */
export function filterPackageGroups(groups: PackageGroup[], searchQuery: string): PackageGroup[] {
  if (!searchQuery.trim()) {
    return groups
  }

  const query = searchQuery.toLowerCase()
  return groups.filter((group) => group.name.toLowerCase().includes(query))
}
