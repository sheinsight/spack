import { useTransition } from 'react'
import { Tabs, TabsContent } from '@/components/ui/tabs'
import { Summary } from '@/components/summary'
import { PackageAnalysis } from '@/components/package-analysis'
import { OptimizationSuggestions } from '@/components/optimization-suggestions'
import { DuplicationAnalysis } from '@/components/duplication-analysis'
import { ChunkViewer } from '@/components/chunk-viewer'
import { ModuleViewer } from '@/components/module-viewer'
import { ModuleRelations } from '@/components/module-relations'
import { FileUploadArea } from '@/components/file-upload-area'
import { AppHeader } from '@/components/app-header'
import { navigateTo, type TabId } from '@/lib/navigation'
import { exportToMarkdown } from '@/lib/utils/export-utils'
import { buildAIFixPrompt } from '@/lib/utils/ai-prompt'
import { useTheme } from '@/hooks/use-theme'
import { useNavigation } from '@/hooks/use-navigation'
import { useBundleData } from '@/hooks/use-bundle-data'
import { useFileUpload } from '@/hooks/use-file-upload'

export function App() {
  // 自定义 Hooks
  const { theme, toggleTheme } = useTheme()
  const navigationParams = useNavigation()
  const { data, hasData, error, setData, setError } = useBundleData()

  const { handleFileChange, isLoading, handleDrop, handleDragOver } = useFileUpload({
    setData,
    setError,
  })

  // React 19 useTransition：优化 tab 切换性能
  const [isPending, startTransition] = useTransition()

  // activeTab 从 navigationParams 派生，URL 是唯一真实来源
  const activeTab = navigationParams.tab || 'optimization'

  // 处理 tab 切换
  const handleTabChange = (value: string) => {
    startTransition(() => {
      navigateTo(value as TabId)
    })
  }

  // 导出处理函数
  const handleExportMarkdown = () => {
    if (data) exportToMarkdown(data)
  }

  const aiPrompt = data ? buildAIFixPrompt(data) : ''

  // 数据未加载时显示文件上传界面
  if (!data) {
    if (!hasData) {
      return (
        <FileUploadArea
          isLoading={isLoading}
          error={error}
          theme={theme}
          onFileChange={handleFileChange}
          onDrop={handleDrop}
          onDragOver={handleDragOver}
          onToggleTheme={toggleTheme}
        />
      )
    } else {
      // loading spinner
      return (
        <div className="min-h-screen flex items-center justify-center bg-background">
          <div className="flex items-center gap-2 text-sm text-primary">
            <div className="w-6 h-6 border-4 border-primary/30 border-t-primary rounded-full animate-spin" />
            <span className="font-medium">正在加载数据...</span>
          </div>
        </div>
      )
    }
  }

  return (
    <div className="min-h-screen bg-background">
      <Tabs value={activeTab} onValueChange={handleTabChange} className="w-full">
        {/* 固定顶部工具栏 */}
        <AppHeader
          theme={theme}
          onExportMarkdown={handleExportMarkdown}
          aiPrompt={aiPrompt}
          onFileChange={handleFileChange}
          onToggleTheme={toggleTheme}
        />

        {/* 内容区域 */}
        <div className="w-full max-w-7xl mx-auto px-8 py-6">
          {/* 过渡加载指示器 */}
          {isPending && (
            <div className="fixed top-20 right-8 z-50 bg-primary/10 backdrop-blur-sm border border-primary/30 rounded-lg px-4 py-2 shadow-lg animate-fade-in">
              <div className="flex items-center gap-2 text-sm text-primary">
                <div className="w-4 h-4 border-2 border-primary/30 border-t-primary rounded-full animate-spin" />
                <span className="font-medium">加载中...</span>
              </div>
            </div>
          )}

          <TabsContent value="optimization" className="mt-0">
            <OptimizationSuggestions data={data} navigationParams={navigationParams} />
          </TabsContent>

          <TabsContent value="summary" className="mt-0">
            <Summary data={data} navigationParams={navigationParams} />
          </TabsContent>

          <TabsContent value="packages" className="mt-0">
            <PackageAnalysis data={data} navigationParams={navigationParams} />
          </TabsContent>

          <TabsContent value="duplication" className="mt-0">
            <DuplicationAnalysis data={data} />
          </TabsContent>

          <TabsContent value="chunks" className="mt-0">
            <ChunkViewer data={data} navigationParams={navigationParams} />
          </TabsContent>

          <TabsContent value="modules" className="mt-0">
            <ModuleViewer data={data} navigationParams={navigationParams} />
          </TabsContent>

          <TabsContent value="relations" className="mt-0">
            <ModuleRelations data={data} navigationParams={navigationParams} />
          </TabsContent>
        </div>
      </Tabs>
    </div>
  )
}
