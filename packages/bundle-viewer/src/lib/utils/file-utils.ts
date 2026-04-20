/**
 * 文件工具函数
 * 提供文件类型识别、扩展名提取等通用功能
 */

/**
 * 获取文件扩展名
 */
export function getFileExtension(path: string): string | null {
  const match = path.match(/\.([^.]+)$/)
  return match ? match[1] : null
}

/**
 * 资源类型定义
 */
export type AssetType = 'javascript' | 'stylesheet' | 'sourcemap' | 'image' | 'font' | 'json' | 'html' | 'other'

/**
 * 根据文件名判断资源类型
 */
export function getAssetType(fileName: string): AssetType {
  // Sourcemap（优先级最高，避免被 .js.map 识别为 js）
  if (fileName.endsWith('.map')) {
    return 'sourcemap'
  }

  // JavaScript
  if (fileName.endsWith('.js') || fileName.endsWith('.mjs') || fileName.endsWith('.cjs')) {
    return 'javascript'
  }

  // CSS
  if (fileName.endsWith('.css')) {
    return 'stylesheet'
  }

  // 图片
  if (fileName.match(/\.(png|jpg|jpeg|gif|svg|webp|ico|avif)$/i)) {
    return 'image'
  }

  // 字体
  if (fileName.match(/\.(woff|woff2|ttf|eot|otf)$/i)) {
    return 'font'
  }

  // JSON
  if (fileName.endsWith('.json')) {
    return 'json'
  }

  // HTML
  if (fileName.endsWith('.html')) {
    return 'html'
  }

  return 'other'
}

/**
 * 获取资源类型的中文名称
 */
export function getAssetTypeName(type: AssetType): string {
  const names: Record<AssetType, string> = {
    javascript: 'JavaScript',
    stylesheet: '样式表',
    sourcemap: 'Source Map',
    image: '图片',
    font: '字体',
    json: 'JSON',
    html: 'HTML',
    other: '其他',
  }
  return names[type]
}

/**
 * 检查是否为图片文件
 */
export function isImageFile(fileName: string): boolean {
  return getAssetType(fileName) === 'image'
}

/**
 * 检查是否为源码文件
 */
export function isSourceFile(fileName: string): boolean {
  const ext = getFileExtension(fileName)
  if (!ext) return false

  const sourceExtensions = ['js', 'jsx', 'ts', 'tsx', 'mjs', 'cjs', 'vue', 'svelte', 'css', 'scss', 'sass', 'less']
  return sourceExtensions.includes(ext.toLowerCase())
}

/**
 * 检查是否为 node_modules 中的文件
 */
export function isNodeModuleFile(path: string): boolean {
  return path.includes('node_modules')
}
