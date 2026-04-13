/**
 * 文件上传区域组件
 */

import { cn } from '@/lib/utils'

interface FileUploadAreaProps {
  isLoading: boolean
  error: string | null
  theme: 'light' | 'dark'
  onFileChange: (event: React.ChangeEvent<HTMLInputElement>) => void
  onDrop: (event: React.DragEvent<HTMLDivElement>) => void
  onDragOver: (event: React.DragEvent<HTMLDivElement>) => void
  onToggleTheme: () => void
}

export function FileUploadArea({
  isLoading,
  error,
  theme,
  onFileChange,
  onDrop,
  onDragOver,
  onToggleTheme,
}: FileUploadAreaProps) {
  return (
    <div className="flex h-screen items-center justify-center bg-background p-6">
      <div className="text-center space-y-6 max-w-2xl w-full">
        <div>
          <h1 className="text-4xl font-bold mb-2">构建分析工具</h1>
          <p className="text-muted-foreground">构建产物可视化分析</p>
        </div>

        {error && (
          <div className="bg-destructive/10 border border-destructive/30 rounded-lg p-4 text-sm text-destructive">
            <div className="font-medium mb-1">⚠️ 错误</div>
            <div>{error}</div>
          </div>
        )}

        <div
          onDrop={onDrop}
          onDragOver={onDragOver}
          className={cn(
            'border-2 border-dashed rounded-lg p-12 transition-all cursor-pointer',
            isLoading
              ? 'border-primary bg-primary/5 pointer-events-none'
              : error
                ? 'border-destructive/50 hover:border-destructive/70 hover:bg-destructive/5'
                : 'border-muted-foreground/25 hover:border-primary/50 hover:bg-muted/50'
          )}
        >
          <div className="space-y-4">
            {isLoading ? (
              <>
                <div className="text-6xl animate-bounce">⏳</div>
                <div>
                  <p className="text-lg font-medium mb-2">正在解析文件...</p>
                  <p className="text-sm text-muted-foreground">请稍候</p>
                </div>
              </>
            ) : (
              <>
                <div className="text-6xl">📦</div>
                <div>
                  <p className="text-lg font-medium mb-2">拖拽构建统计文件到这里</p>
                  <p className="text-sm text-muted-foreground">
                    或者{' '}
                    <label className="text-primary underline cursor-pointer hover:text-primary/80">
                      浏览文件
                      <input
                        type="file"
                        accept=".json"
                        onChange={onFileChange}
                        className="hidden"
                        disabled={isLoading}
                      />
                    </label>
                  </p>
                </div>
                <div className="pt-4 border-t border-muted-foreground/25 mt-6">
                  <p className="text-xs text-muted-foreground">支持各类构建工具生成的 JSON 格式统计数据</p>
                </div>
              </>
            )}
          </div>
        </div>

        <button
          onClick={onToggleTheme}
          className="px-4 py-2 bg-muted hover:bg-muted/80 rounded-md transition-colors text-sm"
          disabled={isLoading}
        >
          {theme === 'light' ? '🌙 深色模式' : '☀️ 浅色模式'}
        </button>
      </div>
    </div>
  )
}
