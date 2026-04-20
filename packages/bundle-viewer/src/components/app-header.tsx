/**
 * 应用顶部工具栏组件
 */

import { useState } from 'react'
import { TabsList, TabsTrigger } from '@/components/ui/tabs'

interface AppHeaderProps {
  theme: 'light' | 'dark'
  onExportMarkdown: () => void
  aiPrompt: string
  onFileChange: (event: React.ChangeEvent<HTMLInputElement>) => void
  onToggleTheme: () => void
}

export function AppHeader({
  theme,
  onExportMarkdown,
  aiPrompt,
  onFileChange,
  onToggleTheme,
}: AppHeaderProps) {
  const [copied, setCopied] = useState(false)

  const copyAIFixGuide = async () => {
    if (!aiPrompt) return
    const prompt = aiPrompt
    try {
      await navigator.clipboard.writeText(prompt)
      setCopied(true)
      window.setTimeout(() => setCopied(false), 2000)
    } catch {
      const textarea = document.createElement('textarea')
      textarea.value = prompt
      textarea.setAttribute('readonly', 'true')
      textarea.style.position = 'fixed'
      textarea.style.opacity = '0'
      document.body.appendChild(textarea)
      textarea.select()
      document.execCommand('copy')
      document.body.removeChild(textarea)
      setCopied(true)
      window.setTimeout(() => setCopied(false), 2000)
    }
  }

  return (
    <div className="sticky top-0 z-50 bg-background/95 backdrop-blur-sm border-b border-border shadow-sm">
      <div className="w-full max-w-7xl mx-auto px-8 py-3">
        <div className="mb-3 flex items-center justify-between">
          <div className="flex items-baseline gap-3">
            <h1 className="text-2xl font-bold">构建分析工具</h1>
            <p className="text-sm text-muted-foreground">构建分析仪表盘</p>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={onExportMarkdown}
              className="px-3 py-1.5 bg-primary text-primary-foreground hover:bg-primary/90 rounded-md transition-colors text-xs font-medium"
              title="导出分析报告为 Markdown"
            >
              导出报告
            </button>
            <button
              onClick={copyAIFixGuide}
              className="px-3 py-1.5 bg-muted hover:bg-muted/80 rounded-md transition-colors text-xs font-medium disabled:opacity-60 disabled:cursor-not-allowed"
              title={aiPrompt ? '复制 AI 修复指引（基于当前分析数据）' : '加载数据后可复制 AI 修复指引'}
              disabled={!aiPrompt}
            >
              {copied ? '已复制' : '复制 AI 修复指引'}
            </button>
            <label className="cursor-pointer px-3 py-1.5 bg-muted hover:bg-muted/80 rounded-md transition-colors text-xs font-medium">
              加载文件
              <input type="file" accept=".json" onChange={onFileChange} className="hidden" />
            </label>
            <button
              onClick={onToggleTheme}
              className="px-2.5 py-1.5 bg-muted hover:bg-muted/80 rounded-md transition-colors text-xs"
              title="切换深色/浅色模式"
            >
              {theme === 'light' ? '深色' : '浅色'}
            </button>
          </div>
        </div>

        <TabsList className="h-9">
          <TabsTrigger value="optimization" className="text-xs px-3">
            优化建议
          </TabsTrigger>
          <TabsTrigger value="summary" className="text-xs px-3">
            概览
          </TabsTrigger>
          <TabsTrigger value="packages" className="text-xs px-3">
            依赖包
          </TabsTrigger>
          <TabsTrigger value="duplication" className="text-xs px-3">
            重复度
          </TabsTrigger>
          <TabsTrigger value="chunks" className="text-xs px-3">
            代码块
          </TabsTrigger>
          <TabsTrigger value="modules" className="text-xs px-3">
            模块
          </TabsTrigger>
          <TabsTrigger value="relations" className="text-xs px-3">
            依赖关系
          </TabsTrigger>
        </TabsList>
      </div>
    </div>
  )
}
