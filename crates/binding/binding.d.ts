export declare const enum CustomPluginNames {
  DuplicateDependencyPlugin = 'DuplicateDependencyPlugin',
  CaseSensitivePathsPlugin = 'CaseSensitivePathsPlugin',
  BundleAnalyzerPlugin = 'BundleAnalyzerPlugin',
  DeadcodePlugin = 'DeadcodePlugin'
}

export interface JsDuplicateDependencyPluginResp {
  libraryGroups: Array<JsLibraryGroup>
  duration: number
}

export interface JsLibrary {
  dir: string
  name: string
  version: string
}

export interface JsLibraryGroup {
  name: string
  libraries: Array<JsLibrary>
}

export interface RawBundleAnalyzerPluginOpts {

}

export interface RawCaseSensitivePathsPluginOpts {

}

export interface RawDeadcodePluginOpts {

}

export interface RawDuplicateDependencyPluginOpts {
  onDetected?: (response: JsDuplicateDependencyPluginResp) => Promise<void>
}

export declare function registerBundleAnalyzerPlugin(): void

export declare function registerCaseSensitivePathsPlugin(): void

export declare function registerDeadcodePlugin(): void

export declare function registerDuplicateDependencyPlugin(): void
