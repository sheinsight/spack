/**
 * 导航工具库 - 用于页面间跳转和URL状态管理
 */

export type TabId = 'optimization' | 'summary' | 'packages' | 'duplication' | 'chunks' | 'modules' | 'relations'

export interface NavigationParams {
  tab?: TabId
  moduleName?: string // 模块名称（替代旧的 moduleId）
  chunkId?: string
  packageName?: string
  search?: string
  filter?: string
  sortBy?: string
  highlight?: string // 逗号分隔的ID/名称列表，用于高亮多个项
}

/**
 * 构建 URL 查询字符串
 */
export function buildQueryString(params: NavigationParams): string {
  const searchParams = new URLSearchParams()

  Object.entries(params).forEach(([key, value]) => {
    if (value !== undefined && value !== null && value !== '') {
      searchParams.set(key, String(value))
    }
  })

  return searchParams.toString()
}

/**
 * 解析 URL 查询参数
 */
export function parseQueryString(search: string = window.location.search): NavigationParams {
  const params = new URLSearchParams(search)

  return {
    tab: (params.get('tab') as TabId) || undefined,
    moduleName: params.get('moduleName') || undefined,
    chunkId: params.get('chunkId') || undefined,
    packageName: params.get('packageName') || undefined,
    search: params.get('search') || undefined,
    filter: params.get('filter') || undefined,
    sortBy: params.get('sortBy') || undefined,
    highlight: params.get('highlight') || undefined,
  }
}

/**
 * 导航到指定 tab 并设置参数
 * @param tab - 目标tab
 * @param additionalParams - 附加参数（不包含tab）
 */
export function navigateTo(tab: TabId, additionalParams?: Omit<NavigationParams, 'tab'>) {
  const params: NavigationParams = {
    tab,
    ...additionalParams,
  }

  const queryString = buildQueryString(params)
  const newUrl = `${window.location.pathname}?${queryString}`

  window.history.pushState(params, '', newUrl)
  window.dispatchEvent(new CustomEvent('navigation', { detail: params }))
}

/**
 * 更新当前 URL 参数（不改变 tab）
 * @param params - 要更新的参数
 */
export function updateParams(params: Partial<NavigationParams>) {
  const currentParams = parseQueryString()
  const newParams = { ...currentParams, ...params }
  const queryString = buildQueryString(newParams)
  const newUrl = `${window.location.pathname}?${queryString}`

  window.history.replaceState(newParams, '', newUrl)
  window.dispatchEvent(new CustomEvent('navigation', { detail: newParams }))
}

/**
 * 从 packageJsonPath 提取包名
 * @param packageJsonPath - package.json 文件路径
 * @returns 包名
 */
export function extractPackageName(packageJsonPath: string): string {
  const match = packageJsonPath.match(/node_modules[/\\]([^/\\]+)/)
  return match ? match[1] : ''
}

/**
 * 安全滚动到元素，带重试机制
 * @param elementId - 目标元素ID
 * @param options - 滚动选项
 * @returns 是否成功滚动
 */
export function scrollToElement(
  elementId: string,
  options?: {
    retries?: number
    delay?: number
    behavior?: ScrollBehavior
  },
): void {
  const { retries = 3, delay = 100, behavior = 'auto' } = options || {}

  let attempt = 0

  const tryScroll = () => {
    const element = document.getElementById(elementId)

    if (element) {
      element.scrollIntoView({
        behavior,
        block: 'nearest', // 改为 nearest 避免滚到导航栏下方
        inline: 'nearest',
      })
      return true
    }

    // 元素未渲染，重试
    if (attempt < retries) {
      attempt++
      setTimeout(tryScroll, delay * attempt) // 递增延迟
    }

    return false
  }

  // 首次尝试立即执行
  setTimeout(tryScroll, 0)
}
