export declare enum CustomPluginNames {
  DuplicateDependencyPlugin = 'DuplicateDependencyPlugin',
  CaseSensitivePathsPlugin = 'CaseSensitivePathsPlugin',
  BundleAnalyzerPlugin = 'BundleAnalyzerPlugin',
  DeadcodePlugin = 'DeadcodePlugin',
  DemoPlugin = 'DemoPlugin'
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

export interface RawBundleAnalyzerPluginOpts {

}

export interface RawCaseSensitivePathsPluginOpts {

}

export interface RawDeadcodePluginOpts {

}

export interface RawDemoPluginOpts {

}

export interface RawDuplicateDependencyPluginOpts {
  onDetected?: (response: JsDuplicateDependencyPluginResp) => void|Promise<void>
}

export declare function registerBundleAnalyzerPlugin(): void

export declare function registerCaseSensitivePathsPlugin(): void

export declare function registerDeadcodePlugin(): void

export declare function registerDemoPlugin(): void

export declare function registerDuplicateDependencyPlugin(): void
