export declare enum CustomPluginNames {
  DuplicateDependencyPlugin = 'DuplicateDependencyPlugin',
  CaseSensitivePathsPlugin = 'CaseSensitivePathsPlugin',
  BundleAnalyzerPlugin = 'BundleAnalyzerPlugin',
  DemoPlugin = 'DemoPlugin',
  UnifiedPlugin = 'UnifiedPlugin'
}

export interface JsAsset {
  name: string
  size: number
  gzipSize?: number
  brotliSize?: number
  chunks: Array<string>
  emitted: boolean
  assetType: string
}

export interface JsBundleAnalyzerPluginResp {
  timestamp: number
  summary: JsSummary
  assets: Array<JsAsset>
  modules: Array<JsModule>
  chunks: Array<JsChunk>
  packages: Array<JsPackage>
}

export interface JsChunk {
  id: string
  names: Array<string>
  size: number
  modules: Array<string>
  entry: boolean
  initial: boolean
  asyncChunks: boolean
  runtime: boolean
  reason: string
  files: Array<string>
  parents: Array<string>
  children: Array<string>
}

export interface JsConcatenatedModuleInfo {
  id: string
  name: string
  size: number
  /** 模块文件类型 */
  moduleType: string
  /** 是否来自 node_modules */
  isNodeModule: boolean
  /** 模块条件名称 */
  nameForCondition: string
  /** 关联的 Package 的 package.json 路径 */
  packageJsonPath?: string
}

export interface JsDuplicateDependencyPluginResp {
  groups: Array<JsLibraryGroup>
  duration: number
}

export interface JsLibrary {
  file: string
  name: string
  version: string
}

export interface JsLibraryGroup {
  name: string
  libs: Array<JsLibrary>
}

export interface JsModule {
  id: string
  name: string
  size: number
  chunks: Array<string>
  moduleKind: string
  moduleType: string
  isNodeModule: boolean
  nameForCondition: string
  concatenatedModules?: Array<JsConcatenatedModuleInfo>
  /**
   * 关联的 Package 的 package.json 路径（唯一标识）
   * 仅三方包模块有值，用于精确匹配对应的 Package
   */
  packageJsonPath?: string
  /** 用户请求路径（如 require('lodash') 中的 'lodash'） */
  userRequest?: string
  /** 原始请求路径（如 loader 链中的完整请求） */
  rawRequest?: string
  /** 当前模块的出站依赖列表（当前模块依赖哪些模块） */
  dependencies?: Array<JsModuleDependency>
  /** 当前模块的入站依赖列表（哪些模块依赖当前模块，为什么被包含） */
  reasons?: Array<JsModuleReason>
}

export interface JsModuleDependency {
  moduleId: string
  moduleName: string
  dependencyId: string
}

export interface JsModuleReason {
  moduleId: string
  moduleName: string
  dependencyId: string
}

export interface JsPackage {
  name: string
  version: string
  size: number
  moduleCount: number
  modules: Array<string>
  packageJsonPath: string
}

export interface JsPerformanceTimings {
  collectAssetsMs: number
  collectModulesMs: number
  collectChunksMs: number
  analyzePackagesMs: number
  totalMs: number
}

export interface JsSummary {
  totalSize: number
  totalGzipSize: number
  totalAssets: number
  totalModules: number
  totalChunks: number
  buildTime: number
  timings: JsPerformanceTimings
}

export interface RawBundleAnalyzerPluginOpts {
  onAnalyzed?: (response: JsBundleAnalyzerPluginResp) => void|Promise<void>
  /** 是否计算 gzip 压缩后的大小（默认：false） */
  gzipAssets?: boolean
  /** 是否计算 brotli 压缩后的大小（默认：false） */
  brotliAssets?: boolean
}

export interface RawCaseSensitivePathsPluginOpts {

}

export interface RawDemoPluginOpts {
  onDetected?: ((err: Error | null, arg: RawDemoResponse) => Promise<undefined>)
}

export interface RawDemoResponse {
  name: string
  age: number
}

export interface RawDuplicateDependencyPluginOpts {
  onDetected?: (response: JsDuplicateDependencyPluginResp) => void|Promise<void>
}

export interface RawEnvironment {
  browser?: boolean
  node?: boolean
  commonjs?: boolean
  es2024?: boolean
  amd?: boolean
  sharedNodeBrowser?: boolean
}

export interface RawOxlintPluginOpts {
  /** runtime 文件的生成目录 , 请保证存在 @@ 的 alias 配置 */
  outputDir: string
  showWarning?: boolean
  /**
   * 是否在有 lint 错误时阻塞构建，默认为 true
   * 设置为 false 时，即使有 lint 错误也继续构建（仅在 dev 模式下推荐）
   */
  failOnError?: boolean
  restrictedImports?: Array<RawRestricted>
  restrictedGlobals?: Array<RawRestricted>
  globals?: Record<string, boolean>
  environments?: RawEnvironment
  configFilePath?: string
}

export interface RawRestricted {
  name: string
  message: string
}

export interface RawUnifiedPluginOpts {
  /** oxlint-loader 的配置 */
  oxlint?: RawOxlintPluginOpts
  /** case-sensitive-paths 的配置 */
  caseSensitive?: RawCaseSensitivePathsPluginOpts
}

export declare function registerBundleAnalyzerPlugin(): void

export declare function registerCaseSensitivePathsPlugin(): void

export declare function registerDemoPlugin(): void

export declare function registerDuplicateDependencyPlugin(): void

export declare function registerUnifiedPlugin(): void
