export declare enum CustomPluginNames {
  DuplicateDependencyPlugin = 'DuplicateDependencyPlugin',
  CaseSensitivePathsPlugin = 'CaseSensitivePathsPlugin',
  BundleAnalyzerPlugin = 'BundleAnalyzerPlugin',
  StyleLoaderPlugin = 'StyleLoaderPlugin'
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

export interface RawDuplicateDependencyPluginOpts {
  onDetected?: (response: JsDuplicateDependencyPluginResp) => void|Promise<void>
}

export interface RawStyleLoaderPluginOpts {
  base?: number
  injectType?: string
  insert?: string
  output: string
  styleTagTransform?: string
  attributes?: Record<string, string>
}

export declare function registerBundleAnalyzerPlugin(): void

export declare function registerCaseSensitivePathsPlugin(): void

export declare function registerDuplicateDependencyPlugin(): void

export declare function registerStyleLoaderPlugin(): void
