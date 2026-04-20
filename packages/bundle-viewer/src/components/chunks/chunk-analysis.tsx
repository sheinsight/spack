import { useEffect, useMemo, useState } from 'react'
import { Card, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import {
  analyzeChunkComposition,
  getCategoryName,
  type ChunkCompositionCategory,
} from '@/lib/chunk-composition-analyzer'
import { scrollToElement, type NavigationParams } from '@/lib/navigation'
import { ChunkCard } from './chunk-card'
import type { StandardBundleData } from '@/types/bundle-data-standard'

interface ChunkAnalysisProps {
  data: StandardBundleData
  navigationParams?: NavigationParams
}

export function ChunkAnalysis({ data, navigationParams }: ChunkAnalysisProps) {
  const [searchQuery, setSearchQuery] = useState(navigationParams?.search || '')
  const [filterType, setFilterType] = useState<'all' | 'entry' | 'async' | 'initial'>('all')
  const [filterComposition, setFilterComposition] = useState<ChunkCompositionCategory | 'all'>('all')

  // 支持多项高亮
  const [highlightedChunkIds, setHighlightedChunkIds] = useState<Set<string>>(() => {
    if (navigationParams?.highlight) {
      return new Set(navigationParams.highlight.split(','))
    }
    if (navigationParams?.chunkId) {
      return new Set([navigationParams.chunkId])
    }
    return new Set()
  })

  // 响应 URL 参数变化
  useEffect(() => {
    if (navigationParams?.search !== undefined) {
      setSearchQuery(navigationParams.search)
    }

    // 更新高亮状态
    if (navigationParams?.highlight) {
      setHighlightedChunkIds(new Set(navigationParams.highlight.split(',')))
    } else if (navigationParams?.chunkId) {
      setHighlightedChunkIds(new Set([navigationParams.chunkId]))
    } else {
      setHighlightedChunkIds(new Set()) // 清除高亮
    }

    // 自动滚动到第一个高亮项
    if (navigationParams?.chunkId || navigationParams?.highlight) {
      const firstId = navigationParams.chunkId || navigationParams.highlight?.split(',')[0]
      if (firstId) {
        scrollToElement(`chunk-${firstId}`)
      }
    }
  }, [navigationParams])

  // 组成分析
  const compositionAnalysis = useMemo(() => analyzeChunkComposition(data), [data])
  const compositionMap = useMemo(() => {
    const map = new Map<string, ChunkCompositionCategory>()
    for (const chunk of compositionAnalysis.chunks) {
      map.set(chunk.chunkId, chunk.category)
    }
    return map
  }, [compositionAnalysis.chunks])

  const filteredChunks = useMemo(() => {
    let chunks = data.chunks

    // 按类型筛选
    if (filterType === 'entry') {
      chunks = chunks.filter((c) => c.type === 'entry')
    } else if (filterType === 'async') {
      chunks = chunks.filter((c) => c.type === 'async')
    } else if (filterType === 'initial') {
      chunks = chunks.filter((c) => c.type === 'initial' || c.type === 'entry')
    }

    // 按组成筛选
    if (filterComposition !== 'all') {
      chunks = chunks.filter((c) => compositionMap.get(c.id) === filterComposition)
    }

    // 按搜索词过滤
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase()
      chunks = chunks.filter(
        (c) =>
          c.names.some((name) => name.toLowerCase().includes(query)) ||
          c.id.toLowerCase().includes(query) ||
          (c.ext?.files || []).some((file) => file.toLowerCase().includes(query)),
      )
    }

    return chunks.toSorted((a, b) => b.size - a.size)
  }, [data.chunks, searchQuery, filterType, filterComposition, compositionMap])

  const stats = useMemo(() => {
    const entryChunks = data.chunks.filter((c) => c.type === 'entry')
    const asyncChunks = data.chunks.filter((c) => c.type === 'async')
    const initialChunks = data.chunks.filter((c) => c.type === 'initial' || c.type === 'entry')
    const runtimeChunks = data.chunks.filter((c) => c.type === 'runtime')

    return {
      total: data.chunks.length,
      entry: entryChunks.length,
      async: asyncChunks.length,
      initial: initialChunks.length,
      runtime: runtimeChunks.length,
    }
  }, [data.chunks])

  return (
    <Card>
      <CardContent className="pt-6 space-y-6">
        {/* 搜索和筛选 */}
        <div className="space-y-4">
          <div className="flex items-center gap-4 flex-wrap">
            <Input
              type="search"
              placeholder="搜索代码块..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="max-w-md"
            />

            <div className="flex gap-2">
              <button
                onClick={() => setFilterType('all')}
                className={`px-3 py-1 text-sm rounded-md transition-colors ${
                  filterType === 'all' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80'
                }`}
              >
                全部 ({stats.total})
              </button>
              <button
                onClick={() => setFilterType('entry')}
                className={`px-3 py-1 text-sm rounded-md transition-colors ${
                  filterType === 'entry' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80'
                }`}
              >
                入口 ({stats.entry})
              </button>
              <button
                onClick={() => setFilterType('initial')}
                className={`px-3 py-1 text-sm rounded-md transition-colors ${
                  filterType === 'initial' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80'
                }`}
              >
                初始 ({stats.initial})
              </button>
              <button
                onClick={() => setFilterType('async')}
                className={`px-3 py-1 text-sm rounded-md transition-colors ${
                  filterType === 'async' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80'
                }`}
              >
                异步 ({stats.async})
              </button>
            </div>

            <span className="text-sm text-muted-foreground ml-auto">{filteredChunks.length} 个代码块</span>
          </div>

          {/* 组成过滤器 */}
          <div className="flex items-center gap-2 flex-wrap">
            <span className="text-sm text-muted-foreground">代码块组成：</span>
            <select
              value={filterComposition}
              onChange={(e) => setFilterComposition(e.target.value as typeof filterComposition)}
              className="px-2 py-1 text-xs rounded-md bg-muted border-0"
            >
              <option value="all">全部</option>
              {compositionAnalysis.distribution.map((dist) => (
                <option key={dist.category} value={dist.category}>
                  {getCategoryName(dist.category)} ({dist.count})
                </option>
              ))}
            </select>
          </div>
        </div>

        {/* Chunk 列表 */}
        <div className="grid gap-3">
          {filteredChunks.map((chunk) => (
            <ChunkCard key={chunk.id} chunk={chunk} data={data} isHighlighted={highlightedChunkIds.has(chunk.id)} />
          ))}
        </div>

        {filteredChunks.length === 0 && (
          <Card>
            <CardContent className="py-12 text-center">
              <p className="text-muted-foreground">未找到匹配 "{searchQuery}" 的代码块</p>
            </CardContent>
          </Card>
        )}
      </CardContent>
    </Card>
  )
}
