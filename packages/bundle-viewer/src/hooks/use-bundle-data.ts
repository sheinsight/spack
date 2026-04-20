/**
 * Bundle 数据加载和管理 Hook
 */

import { useEffect, useState } from 'react'
import { adaptToStandardBundleData } from '@/lib/data-adapter'
import type { StandardBundleData } from '@/types/bundle-data-standard'

interface UseBundleDataReturn {
  data: StandardBundleData | null
  hasData: boolean
  error: string | null
  setData: (data: StandardBundleData) => void
  setError: (error: string | null) => void
}

export function useBundleData(): UseBundleDataReturn {
  const [data, setData] = useState<StandardBundleData | null>(null)
  const [error, setError] = useState<string | null>(null)

  // 初始化：从 window.__bundle_viewer_data__ 加载数据
  useEffect(() => {
    if (window.__bundle_viewer_data__) {
      try {
        const standard = adaptToStandardBundleData(window.__bundle_viewer_data__)
        const timer = setTimeout(() => void setData(standard), 500) // 模拟加载延迟，提升用户体验

        if (import.meta.env.DEV) {
          console.log('📦 Bundle Viewer 数据模型', {
            标准格式: '已启用',
            模块数: standard.modules.length,
            代码块数: standard.chunks.length,
            包数: standard.packages.length,
          })
        }

        return () => clearTimeout(timer) // 清理定时器
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : '数据转换失败'
        setError(`数据加载失败：${errorMessage}`)
        console.error('Failed to adapt bundle data:', err)
      }
    }
  }, [])

  return {
    data,
    hasData: !!window.__bundle_viewer_data__,
    error,
    setData,
    setError,
  }
}
