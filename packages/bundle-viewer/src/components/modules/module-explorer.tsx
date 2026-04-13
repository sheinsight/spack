import { useEffect, useMemo, useState } from 'react'
import { Card, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { beautifyModulePath, getModuleName } from '@/lib/utils/path-utils'
import { scrollToElement, type NavigationParams } from '@/lib/navigation'
import { ModuleCard } from './module-card'
import { cn } from '@/lib/utils'
import type { StandardBundleData } from '@/types/bundle-data-standard'

interface ModuleExplorerProps {
  data: StandardBundleData
  navigationParams?: NavigationParams
  referenceFilter?: 'all' | 'referenced' | 'unreferenced'
}

export function ModuleExplorer({ data, navigationParams, referenceFilter = 'all' }: ModuleExplorerProps) {
  const [searchQuery, setSearchQuery] = useState(navigationParams?.search || '')
  const [filterType, setFilterType] = useState<'all' | 'npm' | 'source'>('all')
  const [filterModuleType, setFilterModuleType] = useState<string>('all')
  const [sortBy, setSortBy] = useState<'size' | 'name' | 'ref'>('size')
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('desc')
  const [highlightedModuleName, setHighlightedModuleName] = useState<string | undefined>(navigationParams?.moduleName)

  const baseModules = useMemo(() => {
    if (referenceFilter === 'referenced') {
      return data.modules.filter((module) => module.chunkIds.length > 0)
    }
    if (referenceFilter === 'unreferenced') {
      return data.modules.filter((module) => module.chunkIds.length === 0)
    }
    return data.modules
  }, [data.modules, referenceFilter])

  // 响应 URL 参数变化
  useEffect(() => {
    if (navigationParams?.search !== undefined) {
      setSearchQuery(navigationParams.search)
    }

    if (navigationParams?.moduleName !== undefined) {
      setHighlightedModuleName(navigationParams.moduleName)
      // 使用新的滚动函数
      scrollToElement(`module-${navigationParams.moduleName}`)
    } else {
      // 清除高亮
      setHighlightedModuleName(undefined)
    }
  }, [navigationParams])

  // 获取所有模块类型
  const moduleTypes = useMemo(() => {
    const types = new Set<string>()
    for (const module of baseModules) {
      types.add(module.type)
    }
    return Array.from(types).toSorted()
  }, [baseModules])

  const filteredModules = useMemo(() => {
    let modules = baseModules

    // 按类型筛选
    if (filterType === 'npm') {
      modules = modules.filter((m) => m.isNodeModule)
    } else if (filterType === 'source') {
      modules = modules.filter((m) => !m.isNodeModule)
    }

    // 按 moduleType 筛选
    if (filterModuleType !== 'all') {
      modules = modules.filter((m) => m.type === filterModuleType)
    }

    // 按搜索词过滤
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase()
      modules = modules.filter((m) => {
        const [mName] = getModuleName(m.name, data.modules)
        return (
          mName.toLowerCase().includes(query) ||
          m.name?.toLowerCase().includes(query) ||
          beautifyModulePath(mName).toLowerCase().includes(query)
        )
      })
    }

    // 排序
    const direction = sortDirection === 'asc' ? 1 : -1
    if (sortBy === 'size') {
      modules = modules.toSorted((a, b) => (a.size - b.size) * direction)
    } else if (sortBy === 'name') {
      modules = modules.toSorted((a, b) => {
        const [aName] = getModuleName(a.name, data.modules)
        const [bName] = getModuleName(b.name, data.modules)
        return aName.localeCompare(bName) * direction
      })
    } else if (sortBy === 'ref') {
      modules = modules.toSorted((a, b) => (a.chunkIds.length - b.chunkIds.length) * direction)
    }

    return modules
  }, [baseModules, searchQuery, filterType, filterModuleType, data, sortBy, sortDirection])

  const stats = useMemo(() => {
    const npmModules = baseModules.filter((m) => m.isNodeModule)
    const sourceModules = baseModules.filter((m) => !m.isNodeModule)
    const npmSize = npmModules.reduce((sum, m) => sum + m.size, 0)
    const sourceSize = sourceModules.reduce((sum, m) => sum + m.size, 0)

    return {
      total: baseModules.length,
      npm: npmModules.length,
      source: sourceModules.length,
      npmSize,
      sourceSize,
      totalSize: npmSize + sourceSize,
    }
  }, [baseModules])

  return (
    <Card>
      <CardContent className="pt-6 space-y-6">
        {/* 搜索和筛选 */}
        <div className="space-y-4">
          <div className="flex items-center gap-4 flex-wrap">
            <Input
              type="search"
              placeholder="搜索模块..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="max-w-md"
            />

            <div className="flex gap-2">
              <button
                onClick={() => setFilterType('all')}
                className={cn(
                  'px-3 py-1 text-sm rounded-md transition-colors',
                  filterType === 'all' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80',
                )}
              >
                全部 ({stats.total})
              </button>
              <button
                onClick={() => setFilterType('npm')}
                className={cn(
                  'px-3 py-1 text-sm rounded-md transition-colors',
                  filterType === 'npm' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80',
                )}
              >
                第三方代码 ({stats.npm})
              </button>
              <button
                onClick={() => setFilterType('source')}
                className={cn(
                  'px-3 py-1 text-sm rounded-md transition-colors',
                  filterType === 'source' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80',
                )}
              >
                项目代码 ({stats.source})
              </button>
            </div>

            <div className="flex gap-2 ml-auto">
              <select
                value={sortBy}
                onChange={(e) => setSortBy(e.target.value as never)}
                className="px-3 py-1 text-sm rounded-md bg-muted border-0"
              >
                <option value="size">按大小</option>
                <option value="name">按名称排序</option>
                <option value="ref">按被引用数排序</option>
              </select>
              <select
                value={sortDirection}
                onChange={(e) => setSortDirection(e.target.value as typeof sortDirection)}
                className="px-3 py-1 text-sm rounded-md bg-muted border-0"
              >
                <option value="desc">降序</option>
                <option value="asc">升序</option>
              </select>
            </div>
          </div>

          {/* 模块类型过滤器 */}
          <div className="flex items-center gap-2 flex-wrap">
            <span className="text-sm text-muted-foreground">模块类型：</span>
            <button
              onClick={() => setFilterModuleType('all')}
              className={cn(
                'px-2 py-1 text-xs rounded-md transition-colors',
                filterModuleType === 'all' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80',
              )}
            >
              全部
            </button>
            {moduleTypes.map((type) => (
              <button
                key={type}
                onClick={() => setFilterModuleType(type)}
                className={cn(
                  'px-2 py-1 text-xs rounded-md transition-colors',
                  filterModuleType === type ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80',
                )}
              >
                {type}
              </button>
            ))}
          </div>
        </div>

        <div className="flex items-center justify-between">
          <span className="text-sm text-muted-foreground">{filteredModules.length} 个模块</span>
        </div>

        {/* 模块列表 */}
        <div className="grid gap-3">
          {filteredModules.slice(0, 100).map((module) => (
            <ModuleCard
              key={module.name}
              module={module}
              data={data}
              isHighlighted={module.name === highlightedModuleName}
            />
          ))}
        </div>

        {filteredModules.length > 100 && (
          <Card>
            <CardContent className="py-6 text-center">
              <p className="text-muted-foreground">
                显示前 100 个模块，共 {filteredModules.length} 个。使用搜索筛选结果
              </p>
            </CardContent>
          </Card>
        )}

        {filteredModules.length === 0 && (
          <Card>
            <CardContent className="py-12 text-center">
              <p className="text-muted-foreground">未找到匹配 "{searchQuery}" 的模块</p>
            </CardContent>
          </Card>
        )}
      </CardContent>
    </Card>
  )
}
