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
}

export interface JsPackage {
  name: string
  version: string
  size: number
  moduleCount: number
  modules: Array<string>
}

export interface JsSummary {
  totalSize: number
  totalGzipSize: number
  totalBrotliSize: number
  totalAssets: number
  totalModules: number
  totalChunks: number
  buildTime: number
}

export interface RawBundleAnalyzerPluginOpts {
  onAnalyzed?: (response: JsBundleAnalyzerPluginResp) => void|Promise<void>
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
