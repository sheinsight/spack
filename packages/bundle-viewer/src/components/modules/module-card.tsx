import { useMemo, useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { beautifyModulePath, getModuleName } from '@/lib/utils/path-utils'
import { formatBytes, getSizeBadgeVariant } from '@/lib/utils/format'
import { getFileExtension } from '@/lib/utils/file-utils'
import { BADGE_STYLES } from '@/lib/constants'
import { navigateTo } from '@/lib/navigation'

import type { StandardBundleData, Module, Chunk } from '@/types/bundle-data-standard'

interface ModuleCardProps {
  module: Module
  data: StandardBundleData
  isHighlighted?: boolean
}

export function ModuleCard({ module, data, isHighlighted }: ModuleCardProps) {
  const [expanded, setExpanded] = useState(false)
  const [moduleName, fullPathWithLoader] = getModuleName(module.name, data.modules)

  // 直接使用标准结构的 Map
  const chunkMap = data.chunkMap

  const chunks = useMemo(() => {
    return module.chunkIds.map((chunkId) => chunkMap.get(chunkId)).filter(Boolean) as Chunk[]
  }, [module.chunkIds, chunkMap])

  const fileExt = getFileExtension(moduleName)

  return (
    <Card id={`module-${module.name}`} className={isHighlighted ? 'ring-2 ring-primary shadow-lg' : ''}>
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between gap-2">
          <div className="flex-1 min-w-0">
            <CardTitle className="text-sm font-mono truncate" title={fullPathWithLoader}>
              {beautifyModulePath(moduleName)}
            </CardTitle>
            <p className="mb-1 flex items-start gap-2 opacity-30">
              <span className="text-muted-foreground text-nowrap">完整路径:</span>
              <span className="font-medium" title={moduleName}>
                {moduleName}
              </span>
            </p>
            <div className="flex gap-1.5 mt-2 flex-wrap">
              <Badge variant={getSizeBadgeVariant(module.size)} className={BADGE_STYLES.SIZE}>
                {formatBytes(module.size)}
              </Badge>
              {fileExt && (
                <Badge variant="outline" className={BADGE_STYLES.COMPACT}>
                  .{fileExt}
                </Badge>
              )}
              <Badge variant="outline" className={BADGE_STYLES.COMPACT}>
                {module.type}
              </Badge>
              {module.isNodeModule ? (
                <Badge variant="secondary" className={BADGE_STYLES.COMPACT}>
                  pkg
                </Badge>
              ) : (
                <Badge variant="outline" className={BADGE_STYLES.COMPACT}>
                  src
                </Badge>
              )}
              <Badge variant="default" className={BADGE_STYLES.NUMERIC}>
                {module.chunkIds.length} 个代码块
              </Badge>
            </div>
          </div>
        </div>
      </CardHeader>

      <CardContent className="space-y-2">
        {/* 基本信息 */}

        {/* <p className="mb-1 flex items-start gap-2">
          <span className="text-muted-foreground">模块类型:</span>
          <span className="font-medium" title={module.type}>
            {module.type}
          </span>
        </p> */}

        {/* <p className="mb-1 flex items-start gap-2">
          <span className="text-muted-foreground text-nowrap">完整路径:</span>
          <span className="font-medium" title={moduleName}>
            {moduleName}
          </span>
        </p> */}

        {module.name && module.name !== moduleName && (
          <p className="mb-1 flex items-start gap-2">
            <span className="text-muted-foreground">条件名称:</span>
            <span className="font-medium" title={module.name}>
              {module.name}
            </span>
          </p>
        )}

        {/* Package 信息 */}
        {/* {module.ext?.packageJsonPath && (
          <p className="mb-1 flex items-start gap-2">
            <span className="text-muted-foreground">package.json:</span>
            <span className="font-medium" title={module.ext.packageJsonPath}>
              {module.ext.packageJsonPath}
            </span>
          </p>
        )} */}

        {/* Chunks 信息 */}
        {chunks.length > 0 && (
          <Collapsible open={expanded} onOpenChange={setExpanded}>
            <CollapsibleTrigger asChild>
              <Button variant="ghost" size="sm" className="w-full justify-start px-0">
                {expanded ? '▼' : '▶'} 在 {chunks.length} 个代码块中使用
              </Button>
            </CollapsibleTrigger>
            <CollapsibleContent className="mt-2 space-y-1">
              {chunks.map((chunk) => (
                <div
                  key={chunk.id}
                  className="text-xs bg-muted px-2 py-1.5 rounded flex items-center justify-between gap-2 cursor-pointer hover:bg-muted/80 transition-colors"
                  onClick={() => navigateTo('chunks', { chunkId: chunk.id, moduleName: module.name })}
                  title="点击查看代码块详情"
                >
                  <span className="font-mono truncate" title={chunk.names.join(', ') || chunk.id}>
                    {chunk.names.join(', ') || chunk.id}
                  </span>
                  <div className="flex gap-1.5 shrink-0">
                    {chunk.type === 'entry' && (
                      <Badge variant="outline" className={BADGE_STYLES.COMPACT}>
                        入口
                      </Badge>
                    )}
                    {(chunk.type === 'initial' || chunk.type === 'entry') && (
                      <Badge variant="outline" className={BADGE_STYLES.COMPACT}>
                        初始
                      </Badge>
                    )}
                  </div>
                </div>
              ))}
            </CollapsibleContent>
          </Collapsible>
        )}

        {/* Concatenated Modules */}
        {module.ext?.concatenatedModules && module.ext.concatenatedModules.length > 0 && (
          <div>
            <p className="text-xs text-muted-foreground mb-1">合并的模块 ({module.ext.concatenatedModules.length})：</p>
            <div className="max-h-32 overflow-y-auto space-y-1">
              {module.ext.concatenatedModules.map((cm, idx) => {
                const [cmName, cmFullPath] = getModuleName(cm.name, data.modules)
                return (
                  <div key={idx} className="text-xs bg-muted px-2 py-1 rounded font-mono truncate" title={cmFullPath}>
                    {beautifyModulePath(cmName)} ({formatBytes(cm.size)})
                  </div>
                )
              })}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
