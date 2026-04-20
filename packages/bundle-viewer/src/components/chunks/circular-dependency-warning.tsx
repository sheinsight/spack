/**
 * 循环依赖警告卡片
 */

import { useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { navigateTo } from '@/lib/navigation'
import { cn } from '@/lib/utils'

interface CircularDependencyWarningProps {
  circularPaths: Array<Array<string | number>>
}

export function CircularDependencyWarning({ circularPaths }: CircularDependencyWarningProps) {
  const [showAllCircularPaths, setShowAllCircularPaths] = useState(false)

  if (circularPaths.length === 0) {
    return null
  }

  return (
    <Card className="border-l-4 border-l-destructive">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-base flex items-center gap-2">🔴 检测到循环依赖</CardTitle>
          {circularPaths.length > 3 && (
            <button
              onClick={() => setShowAllCircularPaths(!showAllCircularPaths)}
              className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
            >
              <span className={cn('transition-transform duration-200', showAllCircularPaths && 'rotate-90')}>▶</span>
              <span>{showAllCircularPaths ? '收起' : `展开全部 (${circularPaths.length - 3})`}</span>
            </button>
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        <p className="text-sm text-muted-foreground">循环依赖可能导致加载死锁或性能问题，建议重构代码结构。</p>
        <div className="space-y-2 max-h-80 overflow-y-auto">
          {(showAllCircularPaths ? circularPaths : circularPaths.slice(0, 3)).map((path, i) => (
            <div
              key={i}
              className="p-2 bg-destructive/10 rounded text-sm font-mono break-all cursor-pointer hover:bg-destructive/20 transition-colors"
              onClick={() => {
                const chunkIds = [...path, path[0]].join(',')
                navigateTo('chunks', { chunkId: String(path[0]), highlight: chunkIds })
              }}
              title="点击查看循环依赖中的代码块"
            >
              {path.join(' → ')} → {path[0]}
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}
