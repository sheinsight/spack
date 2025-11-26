export declare enum CustomPluginNames {
  DuplicateDependencyPlugin = 'DuplicateDependencyPlugin',
  CaseSensitivePathsPlugin = 'CaseSensitivePathsPlugin',
  BundleAnalyzerPlugin = 'BundleAnalyzerPlugin',
  DemoPlugin = 'DemoPlugin',
  UnifiedPlugin = 'UnifiedPlugin'
}

export interface JsBundleAnalyzerPluginResp {
  timestamp: number
  buildTime: number
  summary: JsSummaryInfo
  modules: Array<JsModuleInfo>
  chunks: Array<JsChunkInfo>
  dependencyGraph: Array<JsDependencyNode>
  statistics: JsStatisticsInfo
  visualization: JsVisualizationData
}

export interface JsChunkInfo {
  id: string
  name: string
  size: JsSizeInfo
  modules: Array<string>
  isEntry: boolean
  parents: Array<string>
  children: Array<string>
}

export interface JsDependencyEdge {
  moduleId: string
  dependencyType: string
  userRequest: string
}

export interface JsDependencyNode {
  moduleId: string
  dependencies: Array<JsDependencyEdge>
}

export interface JsDuplicateDependencyPluginResp {
  groups: Array<JsLibraryGroup>
  duration: number
}

export interface JsHeatmapNode {
  name: string
  value: number
  path: string
  level: number
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

export interface JsModuleInfo {
  id: string
  name: string
  path: string
  size: JsSizeInfo
  moduleType: string
  source: string
  isEntry: boolean
  dependencies: Array<string>
}

export interface JsSizeInfo {
  original: number
  minified: number
  gzipped: number
}

export interface JsSourceStatistics {
  count: number
  totalSize: JsSizeInfo
}

export interface JsStatisticsInfo {
  byFileType: Record<string, JsTypeStatistics>
  bySource: Record<string, JsSourceStatistics>
  largestModules: Array<JsModuleInfo>
}

export interface JsSummaryInfo {
  totalModules: number
  totalChunks: number
  totalSize: JsSizeInfo
}

export interface JsTreeNode {
  name: string
  size: number
  children?: Array<JsTreeNode>
  path?: string
  moduleType?: string
}

export interface JsTypeStatistics {
  count: number
  totalSize: JsSizeInfo
}

export interface JsVisualizationData {
  treeData: Array<JsTreeNode>
  heatmapData: Array<JsHeatmapNode>
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
