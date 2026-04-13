/**
 * 工具函数统一导出
 */

// 格式化相关
export {
  safeDivide,
  safePercentage,
  safeAverage,
  clamp,
  safeToFixed,
  formatBytes,
  formatTime,
  formatNumber,
  formatPercentage,
  truncateText,
  getSizeBadgeVariant,
} from './format'

// 文件相关
export {
  getFileExtension,
  type AssetType,
  getAssetType,
  getAssetTypeName,
  isImageFile,
  isSourceFile,
  isNodeModuleFile,
} from './file-utils'

// 路径相关
export { getModuleName, beautifyModulePath, getPathCategory } from './path-utils'
