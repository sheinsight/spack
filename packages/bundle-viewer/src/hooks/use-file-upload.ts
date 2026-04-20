/**
 * 文件上传处理 Hook
 */

import { useCallback, useState } from 'react'
import { adaptToStandardBundleData } from '@/lib/data-adapter'
import type { StandardBundleData } from '@/types/bundle-data-standard'

interface UseFileUploadProps {
  setData: (data: StandardBundleData) => void
  setError: (error: string | null) => void
}

export function useFileUpload({ setData, setError }: UseFileUploadProps) {
  const [isLoading, setIsLoading] = useState(false)

  const processFile = useCallback(
    (file: File) => {
      setIsLoading(true)
      setError(null)

      const reader = new FileReader()
      reader.onload = (e) => {
        try {
          const json = JSON.parse(e.target?.result as string)

          // 验证数据结构
          if (!json.modules || !json.chunks || !json.assets) {
            throw new Error('文件格式不正确，缺少必需的字段（modules, chunks, assets）')
          }

          // 转换为标准格式
          const standard = adaptToStandardBundleData(json)
          setData(standard)
          setError(null)

          if (import.meta.env.DEV) {
            console.log('📦 已加载用户上传的构建数据', {
              模块数: standard.modules.length,
              代码块数: standard.chunks.length,
            })
          }
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : '无效的 JSON 文件'
          setError(`解析失败：${errorMessage}。请确保上传的是有效的构建统计数据文件。`)
          console.error('Failed to parse JSON:', error)
        } finally {
          setIsLoading(false)
        }
      }

      reader.onerror = () => {
        setError('文件读取失败，请重试')
        setIsLoading(false)
      }

      reader.readAsText(file)
    },
    [setData, setError, setIsLoading],
  )

  const handleFileChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0]
      if (!file) return
      processFile(file)
    },
    [processFile],
  )

  const handleDrop = useCallback(
    (event: React.DragEvent<HTMLDivElement>) => {
      event.preventDefault()
      const file = event.dataTransfer.files?.[0]
      if (!file) return

      if (!file.name.endsWith('.json')) {
        setError('解析失败：请上传 JSON 文件。')
        return
      }

      processFile(file)
    },
    [processFile, setError],
  )

  const handleDragOver = useCallback((event: React.DragEvent<HTMLDivElement>) => {
    event.preventDefault()
  }, [])

  return {
    handleFileChange,
    handleDrop,
    handleDragOver,
    isLoading,
  }
}
