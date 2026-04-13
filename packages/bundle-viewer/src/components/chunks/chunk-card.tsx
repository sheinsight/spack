import { useMemo, useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { beautifyModulePath, getPathCategory, getModuleName } from '@/lib/utils/path-utils'
import { formatBytes, getSizeBadgeVariant, formatNumber } from '@/lib/utils/format'
import { getFileExtension } from '@/lib/utils/file-utils'
import { BADGE_STYLES } from '@/lib/constants'
import { navigateTo } from '@/lib/navigation'
import type { StandardBundleData, Module, Chunk } from '@/types/bundle-data-standard'
import { cn } from '@/lib/utils'

interface ChunkCardProps {
  chunk: Chunk
  data: StandardBundleData
  isHighlighted?: boolean
}

const getChunkType = (chunk: Chunk) => {
  const types = []
  if (chunk.type === 'entry') types.push('入口')
  if (chunk.type === 'initial' || chunk.type === 'entry') types.push('初始')
  if (chunk.type === 'runtime') types.push('运行时')
  if (chunk.type === 'async') types.push('异步')
  return types
}

export function ChunkCard({ chunk, data, isHighlighted }: ChunkCardProps) {
  const [depsExpanded, setDepsExpanded] = useState(false)
  const [modulesExpanded, setModulesExpanded] = useState(false)

  const modulesInChunk = useMemo(() => {
    return (chunk.moduleIds.map((moduleId) => data.moduleMap.get(moduleId)).filter(Boolean) as Module[]).toSorted(
      (a, b) => b.size - a.size,
    )
  }, [chunk.moduleIds, data.moduleMap])

  const { thirdPartyModules, projectModules, thirdPartySize, projectSize } = useMemo(() => {
    const thirdPartyModules = modulesInChunk.filter((m) => m.isNodeModule)
    const projectModules = modulesInChunk.filter((m) => !m.isNodeModule)
    const thirdPartySize = thirdPartyModules.reduce((sum, m) => sum + m.size, 0)
    const projectSize = projectModules.reduce((sum, m) => sum + m.size, 0)
    return { thirdPartyModules, projectModules, thirdPartySize, projectSize }
  }, [modulesInChunk])

  return (
    <Card id={`chunk-${chunk.id}`} className={isHighlighted ? 'ring-2 ring-primary shadow-lg' : ''}>
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <div className="flex-1">
            <CardTitle className="text-base font-mono">{chunk.names.join(', ') || chunk.id}</CardTitle>
            <CardDescription className="mt-1">
              {formatBytes(chunk.size)} · {formatNumber(chunk.moduleIds.length)} 个模块
            </CardDescription>
          </div>
          <div className="flex gap-1.5 flex-wrap justify-end">
            {getChunkType(chunk).map((type) => (
              <Badge
                key={type}
                variant={type === '入口' || type === '初始' ? 'default' : 'outline'}
                className={BADGE_STYLES.HEADER}
              >
                {type}
              </Badge>
            ))}
          </div>
        </div>
      </CardHeader>

      <CardContent className="space-y-3">
        {/* 代码占比 */}
        <div className="space-y-2">
          <div className="flex items-center justify-between text-sm">
            <span className="text-muted-foreground">代码分布</span>
            <span className="text-xs text-muted-foreground">{formatBytes(chunk.size)} 总计</span>
          </div>
          <div className="h-3 rounded-full overflow-hidden relative bg-muted/30">
            {/* 第三方依赖 - 碧绿色 */}
            <div
              className="absolute top-0 left-0 h-full bg-data-viz-2 transition-all"
              style={{ width: `${chunk.size > 0 ? (thirdPartySize / chunk.size) * 100 : 0}%` }}
              title={`第三方: ${chunk.size > 0 ? ((thirdPartySize / chunk.size) * 100).toFixed(1) : 0}%`}
            />
            {/* 项目源码 - 钴蓝色 */}
            <div
              className="absolute top-0 h-full bg-data-viz-1 transition-all"
              style={{
                left: `${chunk.size > 0 ? (thirdPartySize / chunk.size) * 100 : 0}%`,
                width: `${chunk.size > 0 ? (projectSize / chunk.size) * 100 : 0}%`,
              }}
              title={`项目: ${chunk.size > 0 ? ((projectSize / chunk.size) * 100).toFixed(1) : 0}%`}
            />
          </div>
          <div className="grid grid-cols-2 gap-2 text-xs">
            <div className="flex items-center gap-1.5">
              {/* 第三方依赖图例 - 碧绿色 */}
              <div className="w-2 h-2 rounded bg-data-viz-2 shrink-0" />
              <div className="flex flex-col">
                <span className="text-muted-foreground">
                  <span className="font-medium text-foreground">{formatBytes(thirdPartySize)}</span> 第三方
                </span>
                <span className="text-muted-foreground">{thirdPartyModules.length} 个模块</span>
              </div>
            </div>
            <div className="flex items-center gap-1.5">
              {/* 项目源码图例 - 钴蓝色 */}
              <div className="w-2 h-2 rounded bg-data-viz-1 shrink-0" />
              <div className="flex flex-col">
                <span className="text-muted-foreground">
                  <span className="font-medium text-foreground">{formatBytes(projectSize)}</span> 项目
                </span>
                <span className="text-muted-foreground">{projectModules.length} 个模块</span>
              </div>
            </div>
          </div>
        </div>

        {/* 文件信息 */}
        {(chunk.ext?.files?.length ?? 0) > 0 && (
          <div>
            <p className="text-sm text-muted-foreground mb-1">输出文件：</p>
            <div className="space-y-1">
              {(chunk.ext?.files || []).map((file) => (
                <p key={file} className="text-xs font-mono bg-muted px-2 py-1 rounded">
                  {file}
                </p>
              ))}
            </div>
          </div>
        )}

        {/* 依赖关系 */}
        {(chunk.parentIds.length > 0 || chunk.childIds.length > 0) && (
          <Collapsible open={depsExpanded} onOpenChange={setDepsExpanded}>
            <CollapsibleTrigger asChild>
              <Button variant="ghost" size="sm" className="w-full justify-start px-0">
                {depsExpanded ? '▼' : '▶'} 依赖关系 ({chunk.parentIds.length} 个父级, {chunk.childIds.length} 个子级)
              </Button>
            </CollapsibleTrigger>
            <CollapsibleContent className="mt-2 space-y-2">
              {chunk.parentIds.length > 0 && (
                <div>
                  <p className="text-xs font-medium text-muted-foreground mb-1">父级代码块：</p>
                  <div className="space-y-1">
                    {chunk.parentIds.map((parentId) => {
                      const parent = data.chunkMap.get(parentId)
                      return (
                        <p key={parentId} className="text-xs font-mono bg-muted px-2 py-1 rounded">
                          {parent?.names.join(', ') || parentId}
                        </p>
                      )
                    })}
                  </div>
                </div>
              )}
              {chunk.childIds.length > 0 && (
                <div>
                  <p className="text-xs font-medium text-muted-foreground mb-1">子级代码块：</p>
                  <div className="space-y-1">
                    {chunk.childIds.map((childId) => {
                      const child = data.chunkMap.get(childId)
                      return (
                        <p key={childId} className="text-xs font-mono bg-muted px-2 py-1 rounded">
                          {child?.names.join(', ') || childId}
                        </p>
                      )
                    })}
                  </div>
                </div>
              )}
            </CollapsibleContent>
          </Collapsible>
        )}

        {/* 模块列表 */}
        {modulesInChunk.length > 0 && (
          <Collapsible open={modulesExpanded} onOpenChange={setModulesExpanded}>
            <CollapsibleTrigger asChild>
              <Button variant="ghost" size="sm" className="w-full justify-start px-0">
                {modulesExpanded ? '▼' : '▶'} 查看 {modulesInChunk.length} 个模块
              </Button>
            </CollapsibleTrigger>
            <CollapsibleContent className="mt-2 space-y-1 max-h-60 overflow-y-auto">
              {modulesInChunk.map((module) => {
                const [moduleName, fullPathWithLoader] = getModuleName(module.name, data.modules)
                const category = getPathCategory(moduleName)
                const fileExt = getFileExtension(moduleName)
                return (
                  <div
                    key={module.name}
                    className="text-xs p-1.5 rounded bg-background/50 font-mono flex items-center gap-2 cursor-pointer hover:bg-muted/80 transition-colors"
                    onClick={() => navigateTo('modules', { moduleName: module.name, chunkId: chunk.id })}
                    title="点击查看模块详情"
                  >
                    <span
                      className={cn(
                        'flex-1 truncate',
                        category === 'project' ? 'text-foreground' : 'text-muted-foreground'
                      )}
                      title={fullPathWithLoader}
                    >
                      {beautifyModulePath(moduleName)}
                    </span>
                    <div className="flex gap-1.5 shrink-0 items-center">
                      {fileExt && (
                        <Badge variant="outline" className={BADGE_STYLES.COMPACT}>
                          .{fileExt}
                        </Badge>
                      )}
                      {module.isNodeModule && (
                        <Badge variant="secondary" className={BADGE_STYLES.COMPACT}>
                          pkg
                        </Badge>
                      )}
                      {module.type && (
                        <Badge variant="outline" className={BADGE_STYLES.COMPACT}>
                          {module.type}
                        </Badge>
                      )}
                      <Badge variant={getSizeBadgeVariant(module.size)} className={BADGE_STYLES.SIZE}>
                        {formatBytes(module.size)}
                      </Badge>
                    </div>
                  </div>
                )
              })}
            </CollapsibleContent>
          </Collapsible>
        )}

        {chunk.ext?.reason && (
          <div>
            <p className="text-xs text-muted-foreground">
              <span className="font-medium">原因：</span> {chunk.ext.reason}
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
