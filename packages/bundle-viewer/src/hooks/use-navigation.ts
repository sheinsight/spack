/**
 * 导航状态管理 Hook
 * 处理 URL 同步和浏览器历史记录
 */

import { useEffect, useState } from 'react'
import { parseQueryString, type NavigationParams } from '@/lib/navigation'

export function useNavigation() {
  const [navigationParams, setNavigationParams] = useState<NavigationParams>(() => {
    return parseQueryString(window.location.search)
  })

  // 监听 URL 变化（浏览器前进/后退）
  useEffect(() => {
    const handlePopState = () => {
      const params = parseQueryString(window.location.search)
      setNavigationParams(params)
    }

    window.addEventListener('popstate', handlePopState)
    return () => window.removeEventListener('popstate', handlePopState)
  }, [])

  // 监听自定义 navigation 事件
  useEffect(() => {
    const handleNavigation = (e: Event) => {
      const customEvent = e as CustomEvent<NavigationParams>
      setNavigationParams(customEvent.detail)
    }

    window.addEventListener('navigation', handleNavigation)
    return () => window.removeEventListener('navigation', handleNavigation)
  }, [])

  return navigationParams
}
