/**
 * 数据适配器
 *
 * 将现有的 BundleData（原始格式）转换为标准化的 StandardBundleData
 */

import type { BundleData } from '@/types/bundle-data'
import type {
  StandardBundleData,
  Module,
  Chunk,
  Package,
  Asset,
  ModuleType,
  ChunkType,
} from '@/types/bundle-data-standard'

/**
 * 推断模块类型
 */
function inferModuleType(moduleKind: string, name: string): ModuleType {
  // 基于 模块类型 推断
  if (moduleKind.includes('json')) return 'json'
  if (moduleKind.includes('css')) return 'css'
  if (moduleKind.includes('asset')) return 'asset'
  if (moduleKind.includes('wasm')) return 'wasm'

  // 基于文件扩展名推断
  const ext = name.split('.').pop()?.toLowerCase() || ''

  if (ext === 'json') return 'json'
  if (ext === 'wasm') return 'wasm'

  if (['css', 'scss', 'less', 'sass', 'styl'].includes(ext)) return 'css'

  if (['woff', 'woff2', 'ttf', 'eot'].includes(ext)) return 'asset'
  if (['jpg', 'jpeg', 'png', 'gif', 'svg', 'webp', 'ico'].includes(ext)) return 'asset'
  if (['mp4', 'flv', '3gp', 'm4v', 'mov', 'avi', 'mkv', 'dat'].includes(ext)) return 'asset'
  if (['mp3', 'wav', 'wma', 'ape', 'm4a', 'ogg', 'aac', 'flac'].includes(ext)) return 'asset'
  if (['xlsx', 'xls', 'doc', 'docx', 'ppt', 'pptx', 'pdf'].includes(ext)) return 'asset'
  if (name.startsWith('data:')) return 'asset'

  // 默认为 JavaScript
  if (['js', 'jsx', 'ts', 'tsx', 'mjs', 'cjs', 'mts', 'cts'].includes(ext)) return 'javascript'

  return 'unknown'
}

/**
 * 确定代码块类型（统一判断逻辑）
 */
function determineChunkType(chunk: BundleData['chunks'][0]): ChunkType {
  // 优先级：runtime > entry > initial > async
  if (chunk.runtime) return 'runtime'
  if (chunk.entry) return 'entry'
  if (chunk.initial) return 'initial'
  return 'async'
}

/**
 * 从路径中提取包名（用于第三方模块）
 */
function extractPackageName(module: BundleData['modules'][0], packages: BundleData['packages']): string | undefined {
  if (!module.isNodeModule) return undefined

  // 方法 1: 通过 packageJsonPath 匹配
  if (module.packageJsonPath) {
    const matchedPkg = packages.find((pkg) => pkg.packageJsonPath === module.packageJsonPath)
    if (matchedPkg) return matchedPkg.name
  }

  // 方法 2: 通过模块名称推断（node_modules/包名/...）
  const name = module.nameForCondition
  const nodeModulesMatch = name.match(/node_modules[/\\](@[^/\\]+[/\\][^/\\]+|[^/\\]+)/)
  if (nodeModulesMatch) {
    return nodeModulesMatch[1].replace(/\\/g, '/')
  }

  return undefined
}

function getBaseDir(names: string[]): string {
  if (names.length === 0) return ''

  const splitPaths = names.map((p) => p.split(/[/\\]/))
  const minLength = Math.min(...splitPaths.map((parts) => parts.length))

  const commonParts: string[] = []

  for (let i = 0; i < minLength; i++) {
    const partSet = new Set(splitPaths.map((parts) => parts[i]))
    if (partSet.size === 1) {
      commonParts.push(splitPaths[0][i])
    } else {
      break
    }
  }

  return commonParts.length > 0 ? commonParts.join('/') + '/' : ''
}

/**
 * 适配模块
 */
function adaptModule(
  baseDir: string,
  legacyModule: BundleData['modules'][0],
  moduleIdMap: BundleData['moduleIdMap'],
  packages: BundleData['packages'],
): Module {
  // 获取模块名称（优先使用 nameForCondition，fallback 到 moduleIdMap）
  const rawName = legacyModule.nameForCondition || moduleIdMap[legacyModule.id] || `模块${legacyModule.id}`
  const name = baseDir ? rawName.replace(baseDir, '') : rawName

  // 推断模块类型
  const type = inferModuleType(legacyModule.moduleKind, rawName)

  // 提取包名
  const packageName = extractPackageName(legacyModule, packages)

  // 构建扩展字段
  const ext: Module['ext'] = {}
  if (legacyModule.nameForCondition) {
    ext.nameForCondition = legacyModule.nameForCondition
  }

  if (legacyModule.packageJsonPath !== undefined) {
    ext.packageJsonPath =
      baseDir && legacyModule.packageJsonPath
        ? legacyModule.packageJsonPath.replace(baseDir, '')
        : legacyModule.packageJsonPath
  }

  if (legacyModule.concatenatedModules !== undefined) {
    // 转换 concatenatedModules，添加 name 字段
    ext.concatenatedModules = legacyModule.concatenatedModules
      ? legacyModule.concatenatedModules.map((cm) => ({
          id: cm.id,
          size: cm.size,
          name: cm.nameForCondition,
          isNodeModule: cm.isNodeModule,
          packageJsonPath: baseDir && cm.packageJsonPath ? cm.packageJsonPath.replace(baseDir, '') : cm.packageJsonPath,
        }))
      : null
  }
  if (legacyModule.rawRequest !== undefined) {
    ext.rawRequest = legacyModule.rawRequest
  }
  if (legacyModule.reasons && legacyModule.reasons.length > 0) {
    ext.reasons = legacyModule.reasons
  }

  return {
    id: legacyModule.id,
    name,
    size: legacyModule.size,
    type,
    isNodeModule: legacyModule.isNodeModule,
    chunkIds: legacyModule.chunks,
    packageName,
    ext: Object.keys(ext).length > 0 ? ext : undefined,
  }
}

/**
 * 适配代码块
 */
function adaptChunk(legacyChunk: BundleData['chunks'][0]): Chunk {
  // 确定统一的类型
  const type = determineChunkType(legacyChunk)

  // 构建扩展字段
  const ext: Chunk['ext'] = {}
  if (legacyChunk.files && legacyChunk.files.length > 0) {
    ext.files = legacyChunk.files
  }
  if (legacyChunk.reason) {
    ext.reason = legacyChunk.reason
  }

  return {
    id: legacyChunk.id,
    names: legacyChunk.names,
    size: legacyChunk.size,
    type,
    moduleIds: legacyChunk.modules,
    parentIds: legacyChunk.parents,
    childIds: legacyChunk.children,
    ext: Object.keys(ext).length > 0 ? ext : undefined,
  }
}

/**
 * 适配 npm 包
 */
function adaptPackage(legacyPackage: BundleData['packages'][0]): Package {
  // 构建扩展字段
  const ext: Package['ext'] = {}

  if (legacyPackage.packageJsonPath) {
    ext.packageJsonPath = legacyPackage.packageJsonPath
  }

  return {
    name: legacyPackage.name,
    version: legacyPackage.version,
    size: legacyPackage.size,
    moduleCount: legacyPackage.moduleCount,
    moduleIds: legacyPackage.modules,
    ext: Object.keys(ext).length > 0 ? ext : undefined,
  }
}

/**
 * 适配静态资源
 */
function adaptAsset(legacyAsset: BundleData['assets'][0]): Asset {
  // 构建扩展字段
  const ext: Asset['ext'] = {}

  if (legacyAsset.gzipSize !== null && legacyAsset.gzipSize !== undefined) {
    ext.gzipSize = legacyAsset.gzipSize
  }
  if (legacyAsset.brotliSize !== null && legacyAsset.brotliSize !== undefined) {
    ext.brotliSize = legacyAsset.brotliSize
  }
  if (legacyAsset.emitted !== undefined) {
    ext.emitted = legacyAsset.emitted
  }

  return {
    name: legacyAsset.name,
    size: legacyAsset.size,
    chunkIds: legacyAsset.chunks,
    ext: Object.keys(ext).length > 0 ? ext : undefined,
  }
}

/**
 * 主适配函数：将旧 BundleData 转换为标准 StandardBundleData
 */
export function adaptToStandardBundleData(legacy: BundleData): StandardBundleData {
  const baseDir = getBaseDir(
    legacy.modules
      .flatMap((c) => c.nameForCondition)
      .filter((e): e is string => typeof e === 'string')
      .filter((e) => e.startsWith('/') || e.includes(':\\\\')),
  )

  const modules = legacy.modules.map((m) => adaptModule(baseDir, m, legacy.moduleIdMap, legacy.packages))
  const chunks = legacy.chunks.map(adaptChunk)
  const packages = legacy.packages.map(adaptPackage)
  const assets = legacy.assets.map(adaptAsset)

  // 构建快速查找 Map（使用 ID 作为 key）
  const moduleMap = new Map<number, Module>()
  for (const module of modules) {
    moduleMap.set(module.id, module)
  }

  const chunkMap = new Map<string, Chunk>()
  for (const chunk of chunks) {
    chunkMap.set(chunk.id, chunk)
  }

  const packageMap = new Map<string, Package[]>()
  for (const pkg of packages) {
    if (!packageMap.has(pkg.name)) {
      packageMap.set(pkg.name, [])
    }
    packageMap.get(pkg.name)!.push(pkg)
  }

  // 构建标准数据
  const standardData: StandardBundleData = {
    timestamp: legacy.timestamp,
    summary: {
      totalSize: legacy.summary.totalSize,
      totalGzipSize: legacy.summary.totalGzipSize,
      totalAssets: legacy.summary.totalAssets,
      totalModules: legacy.summary.totalModules,
      totalChunks: legacy.summary.totalChunks,
      buildTime: legacy.summary.buildTime,
      timings: {
        collectAssetsMs: legacy.summary.timings.collectAssetsMs,
        collectModulesMs: legacy.summary.timings.collectModulesMs,
        collectChunksMs: legacy.summary.timings.collectChunksMs,
        analyzePackagesMs: legacy.summary.timings.analyzePackagesMs,
        totalMs: legacy.summary.timings.totalMs,
      },
    },
    modules,
    chunks,
    packages,
    assets,
    moduleMap,
    chunkMap,
    packageMap,
    ext: {
      buildTool: 'lego', // 当前数据主要来自 lego
    },
  }

  // 开发模式日志
  if (import.meta.env.DEV) {
    console.log('✅ 数据已适配到标准格式', {
      modules: modules.length,
      chunks: chunks.length,
      packages: packages.length,
      assets: assets.length,
    })
  }

  return standardData
}
