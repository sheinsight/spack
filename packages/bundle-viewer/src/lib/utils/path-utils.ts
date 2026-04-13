import type { Module } from '@/types/bundle-data-standard'

/**
 * 获取模块的显示名称和完整路径（含loader信息）
 * @param moduleName - 模块名称
 * @param modules - 模块列表
 * @returns [displayName, fullPathWithLoader] - 显示名称和完整路径（含loader）
 */
export function getModuleName(moduleName: string, _modules: Module[]): [string, string] {
  const fullPath = moduleName

  // module.name 即为纯净路径
  return [fullPath, fullPath]
}

/**
 * 美化模块路径，简化显示
 */
export function beautifyModulePath(path: string): string {
  // 处理 asset 路径
  if (path.startsWith('asset|')) {
    const actualPath = path.substring(6)
    return beautifyModulePath(actualPath)
  }

  // 处理 builtin 路径
  if (path.startsWith('builtin:')) {
    return path
  }

  // 处理 node_modules 路径
  const nodeModulesMatch = path.match(/node_modules[/\\](.+)/)

  if (nodeModulesMatch) {
    const relativePath = nodeModulesMatch[1]
    return `📦 ${relativePath.split('node_modules/').at(-1) || relativePath}`
  }

  // 处理项目路径
  if (path.startsWith('./src/') || path.startsWith('src/')) {
    return `📄 ${path.replace(/^\.\//, '')}`
  }

  // 处理相对路径
  if (path.startsWith('./')) {
    return `📄 ${path.substring(2)}`
  }

  // 绝对路径，尝试提取 src 之后的部分
  const srcMatch = path.match(/[/\\]src[/\\](.+)/)
  if (srcMatch) {
    return `📄 src/${srcMatch[1]}`
  }

  // 其他情况，返回文件名
  const fileName = path.split(/[/\\]/).pop() || path
  return fileName
}

/**
 * 对路径进行分类
 */
export function getPathCategory(path: string): 'third-party' | 'project' | 'builtin' | 'asset' {
  if (path.startsWith('asset|')) return 'asset'
  if (path.startsWith('builtin:')) return 'builtin'
  if (path.includes('node_modules')) return 'third-party'
  return 'project'
}
