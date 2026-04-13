import { useMemo, useState } from 'react'
import type { StandardBundleData } from '@/types/bundle-data-standard'
import type { NavigationParams } from '@/lib/navigation'
import { ModuleExplorer } from './modules/module-explorer'
import { ModuleTreeView } from './modules/module-tree-view'
import { Tabs, TabsContent, TabsList, TabsTrigger } from './ui/tabs'
import { Card, CardContent, CardHeader, CardTitle } from './ui/card'
import { Badge } from './ui/badge'
import { formatBytes } from '@/lib/utils/format'

interface ModuleViewerProps {
  data: StandardBundleData
  navigationParams?: NavigationParams
}

// SVG 图标组件 - 与 ChunkViewer 保持一致
function TreeIcon({ className = '' }: { className?: string }) {
  return (
    <svg
      className={className}
      width="14"
      height="14"
      viewBox="0 0 14 14"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        d="M2 2.5L5 5.5L2 8.5M6.5 7H11.5M2 11H11.5"
        stroke="currentColor"
        strokeWidth="1.2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}

function GridIcon({ className = '' }: { className?: string }) {
  return (
    <svg
      className={className}
      width="14"
      height="14"
      viewBox="0 0 14 14"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <rect x="1.5" y="1.5" width="4.5" height="4.5" rx="0.75" stroke="currentColor" strokeWidth="1.2" />
      <rect x="8" y="1.5" width="4.5" height="4.5" rx="0.75" stroke="currentColor" strokeWidth="1.2" />
      <rect x="1.5" y="8" width="4.5" height="4.5" rx="0.75" stroke="currentColor" strokeWidth="1.2" />
      <rect x="8" y="8" width="4.5" height="4.5" rx="0.75" stroke="currentColor" strokeWidth="1.2" />
    </svg>
  )
}

export function ModuleViewer({ data, navigationParams }: ModuleViewerProps) {
  const [referenceFilter, setReferenceFilter] = useState<'all' | 'referenced' | 'unreferenced'>('referenced')

  const stats = useMemo(() => {
    const totalSize = data.modules.reduce((sum, m) => sum + m.size, 0)
    const projectModules = data.modules.filter((m) => !m.isNodeModule)
    const thirdPartyModules = data.modules.filter((m) => m.isNodeModule)
    const projectSize = projectModules.reduce((sum, m) => sum + m.size, 0)
    const thirdPartySize = thirdPartyModules.reduce((sum, m) => sum + m.size, 0)

    return {
      totalModules: data.modules.length,
      totalSize,
      projectModules: projectModules.length,
      projectSize,
      thirdPartyModules: thirdPartyModules.length,
      thirdPartySize,
    }
  }, [data.modules])

  const referencedStats = useMemo(() => {
    const referencedModules = data.modules.filter((module) => module.chunkIds.length > 0)
    const referencedSize = referencedModules.reduce((sum, module) => sum + module.size, 0)
    return {
      count: referencedModules.length,
      size: referencedSize,
    }
  }, [data.modules])

  const unreferencedStats = useMemo(() => {
    const unreferencedModules = data.modules.filter((module) => module.chunkIds.length === 0)
    const unreferencedSize = unreferencedModules.reduce((sum, module) => sum + module.size, 0)
    return {
      count: unreferencedModules.length,
      size: unreferencedSize,
    }
  }, [data.modules])

  return (
    <Tabs defaultValue="tree" className="w-full space-y-4 animate-fade-in">
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-xs font-medium text-muted-foreground">模块总数</CardTitle>
          </CardHeader>
          <CardContent className="pb-3">
            <p className="text-2xl font-bold">{stats.totalModules}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-xs font-medium text-muted-foreground">总体积</CardTitle>
          </CardHeader>
          <CardContent className="pb-3">
            <p className="text-2xl font-bold">{formatBytes(stats.totalSize)}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-xs font-medium text-muted-foreground">项目代码</CardTitle>
          </CardHeader>
          <CardContent className="pb-3">
            <div className="flex items-baseline gap-2">
              <p className="text-2xl font-bold">{stats.projectModules}</p>
              <Badge variant="secondary" className="text-xs">
                {formatBytes(stats.projectSize)}
              </Badge>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-xs font-medium text-muted-foreground">第三方代码</CardTitle>
          </CardHeader>
          <CardContent className="pb-3">
            <div className="flex items-baseline gap-2">
              <p className="text-2xl font-bold">{stats.thirdPartyModules}</p>
              <Badge variant="outline" className="text-xs">
                {formatBytes(stats.thirdPartySize)}
              </Badge>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* 视图切换器 - 紧挨着视图内容上方，左对齐显示 */}
      <div className="flex items-center justify-between gap-3 flex-wrap">
        <TabsList>
          <TabsTrigger value="tree" className="gap-1.5">
            <TreeIcon />
            <span>树形</span>
          </TabsTrigger>
          <TabsTrigger value="list" className="gap-1.5">
            <GridIcon />
            <span>列表</span>
          </TabsTrigger>
        </TabsList>

        <div className="flex items-center gap-2">
          <span className="text-xs text-muted-foreground">引用筛选：</span>
          <button
            onClick={() => setReferenceFilter('all')}
            className={`px-2.5 py-1 text-xs rounded-md transition-colors ${
              referenceFilter === 'all' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80'
            }`}
          >
            全部 ({stats.totalModules}, {formatBytes(stats.totalSize)})
          </button>
          <button
            onClick={() => setReferenceFilter('referenced')}
            className={`px-2.5 py-1 text-xs rounded-md transition-colors ${
              referenceFilter === 'referenced' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80'
            }`}
          >
            被引用 ({referencedStats.count}, {formatBytes(referencedStats.size)})
          </button>
          <button
            onClick={() => setReferenceFilter('unreferenced')}
            className={`px-2.5 py-1 text-xs rounded-md transition-colors ${
              referenceFilter === 'unreferenced' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80'
            }`}
          >
            未被引用 ({unreferencedStats.count}, {formatBytes(unreferencedStats.size)})
          </button>
        </div>
      </div>

      {/* 视图内容 - 根据 TabsList 切换 */}
      <TabsContent value="tree" className="mt-0">
        <ModuleTreeView data={data} navigationParams={navigationParams} referenceFilter={referenceFilter} />
      </TabsContent>

      <TabsContent value="list" className="mt-0">
        <ModuleExplorer data={data} navigationParams={navigationParams} referenceFilter={referenceFilter} />
      </TabsContent>
    </Tabs>
  )
}
