/**
 * 模块-代码块矩阵视图
 * 显示模块与代码块的关联关系
 */

import { memo, useMemo, useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { formatBytes } from '@/lib/utils/format'
import { navigateTo } from '@/lib/navigation'
import { BADGE_STYLES } from '@/lib/constants'

import type { StandardBundleData } from '@/types/bundle-data-standard'

interface ModuleChunkMatrixProps {
  data: StandardBundleData
}

export const ModuleChunkMatrix = memo(function ModuleChunkMatrix({ data }: ModuleChunkMatrixProps) {
  const [searchTerm, setSearchTerm] = useState('')
  const [filterMode, setFilterMode] = useState<'all' | 'referenced' | 'unreferenced'>('referenced')
  const [sortBy, setSortBy] = useState<'size' | 'refs'>('refs')
  const [expandedModules, setExpandedModules] = useState<Set<string>>(new Set())

  // 构建模块-代码块映射
  const moduleChunkMap = useMemo(() => {
    const map = new Map<
      string,
      {
        module: (typeof data.modules)[0]
        chunks: Array<{ id: string | number; name: string; size: number }>
      }
    >()

    for (const module of data.modules) {
      const chunks = module.chunkIds.map((chunkId) => {
        const chunk = data.chunks.find((c) => c.id === chunkId)
        return {
          id: chunkId,
          name: chunk?.names[0] || `#${chunkId}`,
          size: chunk?.size || 0,
        }
      })

      map.set(module.name, { module, chunks })
    }

    return map
  }, [data])

  // 过滤模块
  const filteredModules = useMemo(() => {
    const entries = Array.from(moduleChunkMap.entries())
    const filtered = entries.filter(([, { chunks }]) => {
      if (filterMode === 'referenced') return chunks.length > 0
      if (filterMode === 'unreferenced') return chunks.length === 0
      return true
    })

    const searched = !searchTerm
      ? filtered
      : filtered.filter(([name]) => name.toLowerCase().includes(searchTerm.toLowerCase()))

    const sorted = searched.toSorted(([, a], [, b]) => {
      if (sortBy === 'size') {
        return b.module.size - a.module.size
      }
      return b.module.chunkIds.length - a.module.chunkIds.length
    })

    return sorted.slice(0, 100)
  }, [moduleChunkMap, searchTerm, filterMode, sortBy])

  // 统计信息
  const stats = useMemo(() => {
    const totalModules = data.modules.length
    const totalChunks = data.chunks.length
    const avgChunksPerModule = data.modules.reduce((sum, m) => sum + m.chunkIds.length, 0) / totalModules
    const referencedModules = data.modules.filter((m) => m.chunkIds.length > 0).length
    const unreferencedModules = totalModules - referencedModules

    return {
      totalModules,
      totalChunks,
      avgChunksPerModule,
      referencedModules,
      unreferencedModules,
    }
  }, [data])

  const topModules = useMemo(() => {
    return [...data.modules].toSorted((a, b) => b.chunkIds.length - a.chunkIds.length || b.size - a.size).slice(0, 6)
  }, [data.modules])

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm">影响面排名（按引用代码块数）</CardTitle>
          <CardDescription>用于识别“改动牵一发动全身”的模块</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2">
          {topModules.map((module) => (
            <div
              key={module.name}
              className="flex items-center justify-between gap-3 rounded-md border px-3 py-2 text-xs cursor-pointer"
            >
              <button
                className="min-w-0 flex-1 truncate cursor-pointer text-left font-mono hover:text-primary transition-colors"
                title={module.name}
                onClick={() => navigateTo('modules', { moduleName: module.name })}
              >
                {module.name}
              </button>
              <div className="flex items-center gap-1">
                <Badge variant="outline" className={BADGE_STYLES.NUMERIC}>
                  {module.chunkIds.length} 块
                </Badge>
                <Badge variant="outline" className={BADGE_STYLES.SIZE}>
                  {formatBytes(module.size)}
                </Badge>
              </div>
            </div>
          ))}
        </CardContent>
      </Card>

      {/* 搜索和列表 */}
      <Card>
        <CardHeader>
          <div className="space-y-3">
            <div>
              <CardTitle>模块-代码块 映射表</CardTitle>
              <CardDescription>查看每个模块被哪些代码块引用</CardDescription>
            </div>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-4 flex-1">
                <Input
                  placeholder="搜索模块名..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="max-w-md"
                />
                <div className="flex gap-2 items-center">
                  <button
                    onClick={() => setFilterMode('all')}
                    className={`px-3 py-1 text-sm rounded-md transition-colors ${
                      filterMode === 'all' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80'
                    }`}
                  >
                    全部 ({stats.totalModules})
                  </button>
                  <button
                    onClick={() => setFilterMode('referenced')}
                    className={`px-3 py-1 text-sm rounded-md transition-colors ${
                      filterMode === 'referenced' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80'
                    }`}
                  >
                    被引用 ({stats.referencedModules})
                  </button>
                  <button
                    onClick={() => setFilterMode('unreferenced')}
                    className={`px-3 py-1 text-sm rounded-md transition-colors ${
                      filterMode === 'unreferenced'
                        ? 'bg-primary text-primary-foreground'
                        : 'bg-muted hover:bg-muted/80'
                    }`}
                  >
                    未被引用 ({stats.unreferencedModules})
                  </button>
                  <span className="text-sm text-muted-foreground ml-auto">{filteredModules.length} 个模块</span>
                </div>
              </div>
              <select
                value={sortBy}
                onChange={(e) => setSortBy(e.target.value as typeof sortBy)}
                className="px-3 py-1 text-sm rounded-md bg-muted border-0"
              >
                <option value="refs">按被引用数</option>
                <option value="size">按大小</option>
              </select>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-2 max-h-150 overflow-y-auto">
            {filteredModules.map(([moduleName, { module, chunks }]) => {
              const isExpanded = expandedModules.has(moduleName)
              const needsCollapse = chunks.length > 20
              const displayCount = needsCollapse ? (isExpanded ? chunks.length : 20) : chunks.length
              return (
                <div key={moduleName} className="p-3 rounded-lg border hover:bg-muted/50 transition-colors">
                  <div className="flex items-center justify-between gap-3">
                    <div className="min-w-0 flex-1 flex items-center gap-2">
                      <button
                        className="min-w-0 flex-1 truncate text-left font-mono text-sm hover:text-primary transition-colors"
                        title={moduleName}
                        onClick={() => navigateTo('modules', { moduleName })}
                      >
                        {moduleName}
                      </button>
                      <div className="flex items-center gap-1 shrink-0">
                        <Badge variant="outline" className={BADGE_STYLES.SIZE}>
                          {formatBytes(module.size)}
                        </Badge>
                        <Badge variant="outline" className={BADGE_STYLES.NUMERIC}>
                          {module.chunkIds.length} 块
                        </Badge>
                        {module.isNodeModule && (
                          <Badge variant="secondary" className={BADGE_STYLES.COMPACT}>
                            npm
                          </Badge>
                        )}
                      </div>
                    </div>
                  </div>
                  <div className="mt-2 text-xs text-muted-foreground flex items-center gap-2">
                    <Badge variant="secondary" className={BADGE_STYLES.COMPACT}>
                      引用于
                    </Badge>
                    {chunks.length === 0 ? (
                      <Badge variant="outline" className={BADGE_STYLES.COMPACT}>
                        未被引用
                      </Badge>
                    ) : (
                      <div className="flex flex-wrap items-center gap-1">
                        {chunks.slice(0, displayCount).map((chunk) => (
                          <Badge
                            key={chunk.id}
                            variant="outline"
                            className="cursor-pointer hover:bg-primary/10"
                            onClick={() => navigateTo('chunks', { chunkId: String(chunk.id) })}
                            title={`${chunk.name}\n${formatBytes(chunk.size)}`}
                          >
                            {chunk.name}
                          </Badge>
                        ))}
                        {needsCollapse && (
                          <button
                            onClick={() => {
                              const next = new Set(expandedModules)
                              if (isExpanded) {
                                next.delete(moduleName)
                              } else {
                                next.add(moduleName)
                              }
                              setExpandedModules(next)
                            }}
                            className="text-xs text-muted-foreground hover:text-foreground transition-colors"
                          >
                            <span>{isExpanded ? '收起' : `展开 ${chunks.length - 20}`}</span>
                          </button>
                        )}
                      </div>
                    )}
                  </div>
                </div>
              )
            })}
          </div>

          {filteredModules.length === 0 && (
            <div className="text-center py-8 text-muted-foreground">未找到匹配的模块</div>
          )}

          {filteredModules.length >= 100 && (
            <div className="mt-4 text-center text-xs text-muted-foreground">仅显示前 100 个结果</div>
          )}
        </CardContent>
      </Card>
    </div>
  )
})
