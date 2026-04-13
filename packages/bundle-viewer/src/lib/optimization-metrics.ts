import { SIZE_THRESHOLDS } from '@/lib/constants'
import { getModuleName } from '@/lib/utils/path-utils'
import { safePercentage } from '@/lib/utils/format'
import { getMajorVersion, type DuplicatePackage } from '@/lib/optimization-utils'
import type { StandardBundleData } from '@/types/bundle-data-standard'

export function buildOptimizationMetrics(data: StandardBundleData) {
  const packageVersionSetMap = new Map<string, Set<string>>()
  const duplicatePackageMap = new Map<string, DuplicatePackage>()
  let totalPackageSize = 0

  for (const pkg of data.packages) {
    totalPackageSize += pkg.size

    if (!packageVersionSetMap.has(pkg.name)) {
      packageVersionSetMap.set(pkg.name, new Set())
    }
    packageVersionSetMap.get(pkg.name)!.add(pkg.version)

    if (!duplicatePackageMap.has(pkg.name)) {
      duplicatePackageMap.set(pkg.name, {
        name: pkg.name,
        versions: [],
        totalSize: 0,
        potentialSavings: 0,
      })
    }

    const group = duplicatePackageMap.get(pkg.name)!
    group.versions.push({
      version: pkg.version,
      size: pkg.size,
      moduleCount: pkg.moduleCount,
    })
    group.totalSize += pkg.size
  }

  const packageStats = {
    total: packageVersionSetMap.size,
    multiVersion: Array.from(packageVersionSetMap.values()).filter((versions) => versions.size > 1).length,
    totalPackages: data.packages.length,
  }

  const duplicatePackages = Array.from(duplicatePackageMap.values())
    .filter((g) => g.versions.length > 1)
    .map((g) => {
      const byMajor = g.versions.reduce((map, v) => {
        const major = getMajorVersion(v.version)
        if (!map.has(major)) {
          map.set(major, [])
        }
        map.get(major)!.push(v)
        return map
      }, new Map<string, typeof g.versions>())

      let potentialSavings = 0
      let canDedupe = false

      for (const versions of byMajor.values()) {
        if (versions.length > 1) {
          const sortedVersions = versions.toSorted((a, b) => b.size - a.size)
          potentialSavings += sortedVersions.slice(1).reduce((sum, v) => sum + v.size, 0)
          canDedupe = true
        }
      }

      return {
        ...g,
        versions: g.versions.toSorted((a, b) => b.size - a.size),
        potentialSavings,
        canDedupe,
        hasDifferentMajors: byMajor.size > 1,
        majorVersionCount: byMajor.size,
      }
    })
    .toSorted((a, b) => b.potentialSavings - a.potentialSavings)

  const largePackages = data.packages
    .filter((p) => p.size > SIZE_THRESHOLDS.LARGE_PACKAGE)
    .map((p) => ({
      name: p.name,
      version: p.version,
      size: p.size,
      moduleCount: p.moduleCount,
      percentage: safePercentage(p.size, totalPackageSize),
    }))
    .toSorted((a, b) => b.size - a.size)
    .slice(0, 20)

  let totalModuleSize = 0
  for (const module of data.modules) {
    totalModuleSize += module.size
  }

  const largeModules = data.modules
    .filter((m) => m.size > SIZE_THRESHOLDS.LARGE_MODULE)
    .map((m) => {
      const [name] = getModuleName(m.name, data.modules)
      return {
        name: m.name,
        displayName: name,
        size: m.size,
        chunkIds: m.chunkIds,
        type: m.type,
        isNodeModule: m.isNodeModule,
        percentage: safePercentage(m.size, totalModuleSize),
      }
    })
    .toSorted((a, b) => b.size - a.size)
    .slice(0, 20)

  const entryChunks = data.chunks.filter((c) => c.type === 'entry')
  const asyncChunks = data.chunks.filter((c) => c.type === 'async')
  const entrySize = entryChunks.reduce((sum, c) => sum + c.size, 0)
  const asyncSize = asyncChunks.reduce((sum, c) => sum + c.size, 0)
  const totalSize = data.chunks.reduce((sum, c) => sum + c.size, 0)
  const largeEntryChunks = entryChunks.filter((c) => c.size > SIZE_THRESHOLDS.ENTRY_CHUNK_WARNING)
  const tooManyModulesChunks = data.chunks.filter((c) => c.moduleIds.length > 1000)

  const chunkQuality = {
    entrySize,
    asyncSize,
    totalSize,
    entryPercentage: safePercentage(entrySize, totalSize),
    asyncPercentage: safePercentage(asyncSize, totalSize),
    largeEntryChunks,
    tooManyModulesChunks,
    asyncChunkCount: asyncChunks.length,
  }

  return {
    packageStats,
    duplicatePackages,
    largePackages,
    largeModules,
    chunkQuality,
  }
}
