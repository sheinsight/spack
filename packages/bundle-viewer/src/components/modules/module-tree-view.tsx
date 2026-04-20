import { useState, useMemo, useEffect, useRef, useCallback } from 'react'
import { navigateTo, scrollToElement, type NavigationParams } from '@/lib/navigation'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { buildModuleTree, getCompressedPath, searchTree, type TreeNode } from '@/lib/tree-builder'
import { formatBytes, getSizeBadgeVariant } from '@/lib/utils/format'
import { getFileExtension } from '@/lib/utils/file-utils'
import { BADGE_STYLES } from '@/lib/constants'
import { cn } from '@/lib/utils'
import type { StandardBundleData, Chunk } from '@/types/bundle-data-standard'

interface ModuleTreeViewProps {
  data: StandardBundleData
  navigationParams?: NavigationParams
  referenceFilter?: 'all' | 'referenced' | 'unreferenced'
}

interface TreeNodeComponentProps {
  node: TreeNode
  level: number
  sortBy: 'name' | 'size' | 'ref'
  sortDirection: 'asc' | 'desc'
  refCountMap: Map<string, number>
  allChunks: Chunk[]
  highlightedModuleName?: string
  highlightedModulePath?: string // 高亮模块的完整路径（用于路径匹配）
  expandedPaths: Set<string>
}

function TreeNodeComponent({
  node,
  level,
  sortBy,
  sortDirection,
  refCountMap,
  allChunks,
  highlightedModuleName,
  highlightedModulePath,
  expandedPaths,
}: TreeNodeComponentProps) {
  // 如果当前路径在 expandedPaths 中，自动展开
  const shouldAutoExpand = !node.isFile && expandedPaths.has(node.fullPath)

  // 初始化时就检查是否需要展开（level 0 或在 expandedPaths 中）
  const [isExpanded, setIsExpanded] = useState(shouldAutoExpand)

  // 标记是否正在手动收起（避免竞态条件）
  const isManuallyCollapsingRef = useRef(false)

  // 响应 expandedPaths 变化 - 当路径在 expandedPaths 中时自动展开
  // 但只在有选中模块且不是手动收起时才展开（避免收起时的竞态问题）
  useEffect(() => {
    if (shouldAutoExpand && !isExpanded && highlightedModuleName !== undefined && !isManuallyCollapsingRef.current) {
      setIsExpanded(true)
    }

    // 当 highlightedModuleName 变成 undefined 时，清除手动收起标记
    if (highlightedModuleName === undefined) {
      isManuallyCollapsingRef.current = false
    }
  }, [shouldAutoExpand, node.fullPath, expandedPaths, isExpanded, highlightedModuleName])

  const { displayName, targetNode } = useMemo(() => getCompressedPath(node), [node])

  // 统计文件夹子项信息
  const childStats = useMemo(() => {
    if (node.isFile) return null

    let fileCount = 0
    const countFiles = (n: TreeNode) => {
      if (n.isFile) {
        fileCount++
      } else {
        for (const child of n.children.values()) {
          countFiles(child)
        }
      }
    }
    countFiles(targetNode)

    return { fileCount, totalSize: node.size }
  }, [node, targetNode])

  const childNodes = useMemo(() => {
    const children = Array.from(targetNode.children.values())
    const direction = sortDirection === 'asc' ? 1 : -1
    return children.toSorted((a, b) => {
      if (a.isFile !== b.isFile) {
        return a.isFile ? 1 : -1
      }
      if (sortBy === 'name') {
        return a.name.localeCompare(b.name) * direction
      }
      if (sortBy === 'ref') {
        const aRefs = refCountMap.get(a.fullPath) ?? 0
        const bRefs = refCountMap.get(b.fullPath) ?? 0
        return (aRefs - bRefs) * direction
      }
      return (a.size - b.size) * direction
    })
  }, [targetNode, sortBy, sortDirection, refCountMap])

  const hasChildren = childNodes.length > 0

  const handleToggle = () => {
    // 文件节点：跳转到模块详情
    if (node.isFile && node.moduleName !== undefined) {
      navigateTo('modules', { moduleName: node.moduleName })
      setIsExpanded(!isExpanded) // 点击文件时也切换展开状态，显示/隐藏详情
      return
    }

    // 文件夹节点：展开/收起
    if (hasChildren) {
      // 如果正在收起且该文件夹在 expandedPaths 中，清除选中
      if (isExpanded && shouldAutoExpand) {
        isManuallyCollapsingRef.current = true // 标记正在手动收起
        navigateTo('modules') // 清除 moduleId，回到列表页
      }
      setIsExpanded(!isExpanded)
    }
  }

  const getIcon = () => {
    if (node.isFile) {
      return '📄'
    }
    return isExpanded ? '📂' : '📁'
  }

  const getChunks = useCallback(
    (chunkIds: string[]) => {
      return chunkIds.flatMap((chunkId) => {
        const chunk = allChunks.find((c) => c.id === chunkId)
        if (!chunk) return [{ id: chunkId, name: `#${chunkId}` }]
        if (chunk.names && chunk.names.length > 0) return chunk.names.map((name) => ({ id: chunkId, name }))
        return [{ id: chunkId, name: `#${chunkId}` }]
      })
    },
    [allChunks],
  )

  const fileExt = node.isFile ? getFileExtension(node.name) : null
  const refCount = refCountMap.get(node.fullPath) ?? 0
  const moduleChunks = useMemo(() => {
    if (!node.isFile || !node.module) return []
    return getChunks(node.module.chunkIds)
  }, [node.isFile, node.module, getChunks])

  // 使用路径匹配而非 moduleId 匹配（因为多个模块可能有相同路径）
  const isHighlighted = node.isFile && highlightedModulePath && node.fullPath === highlightedModulePath

  return (
    <>
      <div
        id={node.isFile && node.module ? `module-${node.module.name}` : undefined}
        className={cn(
          'flex items-center gap-2 py-1.5 px-2 hover:bg-muted/50 rounded cursor-pointer group',
          node.isFile
            ? 'text-sm'
            : 'font-medium text-sm bg-background/95 backdrop-blur-sm z-10 border-b border-border/50',
          isHighlighted && 'ring-2 ring-primary bg-primary/5',
        )}
        style={{
          paddingLeft: `${level * 16 + 8}px`,
          position: node.isFile ? undefined : 'sticky',
          top: node.isFile ? undefined : `${level * 32}px`,
          zIndex: node.isFile ? undefined : 100 - level, // 层级越浅，z-index 越高
        }}
        onClick={handleToggle}
      >
        <span className="shrink-0 select-none">{hasChildren ? (isExpanded ? '▼' : '▶') : ' '}</span>
        <span className="shrink-0 mr-1">{getIcon()}</span>
        <span className={cn('flex-1 truncate', node.isFile && 'font-mono text-xs')} title={displayName}>
          {displayName}
        </span>

        <div className="flex gap-1.5 shrink-0 items-center">
          {/* 文件夹显示子项统计 */}
          {!node.isFile && childStats && (
            <span className="text-muted-foreground text-[0.65rem] font-mono whitespace-nowrap">
              ({childStats.fileCount})
            </span>
          )}
          {node.isFile && fileExt && (
            <Badge variant="outline" className={BADGE_STYLES.COMPACT}>
              .{fileExt}
            </Badge>
          )}
          {node.isFile && node.module && (
            <>
              {node.module.isNodeModule && (
                <Badge variant="secondary" className={BADGE_STYLES.COMPACT}>
                  pkg
                </Badge>
              )}
              {node.module.type && (
                <Badge variant="outline" className={BADGE_STYLES.COMPACT}>
                  {node.module.type}
                </Badge>
              )}
            </>
          )}
          <Badge variant="outline" className={BADGE_STYLES.COMPACT} title="被引用次数">
            {refCount === 0 ? '未被引用' : `${refCount} 引用`}
          </Badge>
          <Badge
            variant={getSizeBadgeVariant(node.size)}
            className={cn(BADGE_STYLES.SIZE, !node.isFile && 'opacity-90')}
            title={node.isFile ? `文件体积: ${formatBytes(node.size)}` : `文件夹总计: ${formatBytes(node.size)}`}
          >
            {formatBytes(node.size)}
          </Badge>
        </div>
      </div>

      {/* 文件详情（展开时显示） */}
      {node.isFile && node.module && isExpanded && (
        <div
          className="text-xs text-muted-foreground py-2 px-2 border-l-2 border-muted ml-4"
          style={{ paddingLeft: `${level * 16 + 24}px` }}
        >
          <div className="space-y-1">
            <div>
              <span className="font-medium">被引用代码块：</span>
              {moduleChunks.length > 0
                ? moduleChunks.map((chunk) => {
                    return (
                      <Badge
                        key={chunk.id}
                        variant="outline"
                        className={cn(BADGE_STYLES.COMPACT, 'cursor-pointer')}
                        onClick={(e) => {
                          e.stopPropagation()
                          navigateTo('chunks', { chunkId: chunk.id })
                        }}
                        title={`跳转到代码块 ${chunk.name}`}
                      >
                        {chunk.name}
                      </Badge>
                    )
                  })
                : '无'}
            </div>
            {node.module.ext?.packageJsonPath && (
              <div className="truncate" title={node.module.ext.packageJsonPath}>
                <span className="font-medium">包：</span>
                {node.module.ext.packageJsonPath}
              </div>
            )}
          </div>
        </div>
      )}

      {/* 子节点 */}
      {isExpanded && hasChildren && (
        <div>
          {childNodes.map((child) => (
            <TreeNodeComponent
              key={child.fullPath}
              node={child}
              level={level + 1}
              sortBy={sortBy}
              sortDirection={sortDirection}
              refCountMap={refCountMap}
              allChunks={allChunks}
              highlightedModuleName={highlightedModuleName}
              highlightedModulePath={highlightedModulePath}
              expandedPaths={expandedPaths}
            />
          ))}
        </div>
      )}
    </>
  )
}

// 递归查找目标模块的路径（返回需要展开的文件夹路径列表）
// 注意：由于多个模块可能有相同的路径（如 Concatenated 模块），使用路径匹配而非 moduleId 匹配
function findModulePathInTree(tree: TreeNode, targetFullPath: string, currentPath: string[] = []): string[] | null {
  // 如果当前节点是文件且路径匹配，返回路径（不包括文件本身）
  if (tree.isFile && tree.fullPath === targetFullPath) {
    return currentPath
  }

  // 如果是文件夹，递归搜索子节点
  if (!tree.isFile) {
    for (const [, childNode] of tree.children) {
      // 将当前节点路径加入路径数组（如果不是根节点）
      const newPath = tree.fullPath ? [...currentPath, tree.fullPath] : currentPath
      const result = findModulePathInTree(childNode, targetFullPath, newPath)
      if (result) {
        return result
      }
    }
  }

  return null
}

export function ModuleTreeView({ data, navigationParams, referenceFilter = 'all' }: ModuleTreeViewProps) {
  const [searchQuery, setSearchQuery] = useState(navigationParams?.search || '')
  const [filterType, setFilterType] = useState<'all' | 'npm' | 'source'>('all')
  const [filterModuleType, setFilterModuleType] = useState<string>('all')
  const [sortBy, setSortBy] = useState<'name' | 'size' | 'ref'>('size')
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('desc')
  const [highlightedModuleName, setHighlightedModuleName] = useState<string | undefined>(navigationParams?.moduleName)
  const [highlightedModulePath, setHighlightedModulePath] = useState<string | undefined>()
  // 需要展开的路径集合（存储完整路径）
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(new Set())

  const baseModules = useMemo(() => {
    if (referenceFilter === 'referenced') {
      return data.modules.filter((module) => module.chunkIds.length > 0)
    }
    if (referenceFilter === 'unreferenced') {
      return data.modules.filter((module) => module.chunkIds.length === 0)
    }
    return data.modules
  }, [data.modules, referenceFilter])

  // 构建完整树（用于路径查找）
  const fullTree = useMemo(() => {
    return buildModuleTree(data.modules)
  }, [data.modules])

  // 构建筛选后的树（用于显示）
  const tree = useMemo(() => {
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

    return buildModuleTree(modules)
  }, [baseModules, filterType, filterModuleType])

  // 获取所有模块类型
  const moduleTypes = useMemo(() => {
    const types = new Set<string>()
    for (const module of baseModules) {
      types.add(module.type)
    }
    return Array.from(types).sort()
  }, [baseModules])

  const moduleTypeStats = useMemo(() => {
    const map = new Map<string, { count: number; size: number }>()
    for (const module of baseModules) {
      const existing = map.get(module.type) ?? { count: 0, size: 0 }
      existing.count += 1
      existing.size += module.size
      map.set(module.type, existing)
    }
    return map
  }, [baseModules])

  // 缓存模块路径查找结果，避免重复计算
  // 使用完整树查找路径（避免筛选器导致找不到）
  const modulePathMap = useMemo(() => {
    const targetName = navigationParams?.moduleName
    if (targetName === undefined || !fullTree) return null

    const targetModule = data.modules.find((m) => m.name === targetName)
    if (!targetModule) {
      return null
    }

    // 将模块的 name 转换为树的 fullPath 格式
    let modulePath = targetModule.name

    // 处理特殊路径前缀
    if (modulePath.startsWith('asset|')) {
      modulePath = modulePath.substring(6)
    }

    // 转换为树的 fullPath 格式（与 tree-builder.ts 保持一致）
    const pathParts = modulePath.split(/[/\\]/).filter(Boolean)
    const targetFullPath = pathParts.join('/')

    const pathToModule = findModulePathInTree(fullTree, targetFullPath)
    if (!pathToModule || pathToModule.length === 0) {
      return null
    }

    const filteredPaths = pathToModule.filter((p) => p !== '')
    return new Set(filteredPaths)
  }, [navigationParams?.moduleName, fullTree, data.modules])

  // 响应 URL 参数变化
  useEffect(() => {
    if (navigationParams?.search !== undefined) {
      setSearchQuery(navigationParams.search)
    }

    if (navigationParams?.moduleName !== undefined) {
      setHighlightedModuleName(navigationParams.moduleName)

      // 设置高亮路径
      const targetModule = data.modules.find((m) => m.name === navigationParams.moduleName)
      if (targetModule) {
        let modulePath = targetModule.name
        if (modulePath.startsWith('asset|')) {
          modulePath = modulePath.substring(6)
        }
        const pathParts = modulePath.split(/[/\\]/).filter(Boolean)
        const fullPath = pathParts.join('/')
        setHighlightedModulePath(fullPath)
      }

      if (modulePathMap) {
        // 检查目标模块是否在筛选后的树中
        const targetModule = data.modules.find((m) => m.name === navigationParams.moduleName)
        const isFilteredOut =
          targetModule &&
          ((filterType === 'npm' && !targetModule.isNodeModule) ||
            (filterType === 'source' && targetModule.isNodeModule) ||
            (filterModuleType !== 'all' && targetModule.type !== filterModuleType) ||
            (referenceFilter === 'referenced' && targetModule.chunkIds.length === 0) ||
            (referenceFilter === 'unreferenced' && targetModule.chunkIds.length > 0))

        // 如果被筛选掉，自动清除筛选器
        if (isFilteredOut) {
          setFilterType('all')
          setFilterModuleType('all')
        }

        setExpandedPaths(modulePathMap)

        // 延迟滚动，等待展开完成
        setTimeout(() => {
          scrollToElement(`module-${navigationParams.moduleName}`, { delay: 150, retries: 5 })
        }, 200)
      }
    } else {
      setHighlightedModuleName(undefined)
      setHighlightedModulePath(undefined)
      setExpandedPaths(new Set())
    }
  }, [navigationParams, modulePathMap, data.modules, filterType, filterModuleType, referenceFilter])

  // 搜索结果
  const searchResults = useMemo(() => {
    if (!searchQuery.trim()) return null
    return searchTree(tree, searchQuery)
  }, [tree, searchQuery])

  const refCountMap = useMemo(() => {
    const map = new Map<string, number>()
    const visit = (node: TreeNode): number => {
      if (node.isFile) {
        const refs = node.module?.chunkIds.length ?? 0
        map.set(node.fullPath, refs)
        return refs
      }
      let total = 0
      for (const child of node.children.values()) {
        total += visit(child)
      }
      map.set(node.fullPath, total)
      return total
    }
    visit(tree)
    return map
  }, [tree])

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

  const rootNodes = useMemo(() => {
    const children = Array.from(tree.children.values())
    const direction = sortDirection === 'asc' ? 1 : -1
    return children.toSorted((a, b) => {
      if (a.isFile !== b.isFile) {
        return a.isFile ? 1 : -1
      }
      if (sortBy === 'name') {
        return a.name.localeCompare(b.name) * direction
      }
      if (sortBy === 'ref') {
        const aRefs = refCountMap.get(a.fullPath) ?? 0
        const bRefs = refCountMap.get(b.fullPath) ?? 0
        return (aRefs - bRefs) * direction
      }
      return (a.size - b.size) * direction
    })
  }, [tree, sortBy, sortDirection, refCountMap])

  return (
    <Card>
      <CardContent className="pt-6 space-y-6">
        {/* 搜索和筛选 */}
        <div className="space-y-4">
          <div className="flex items-center gap-4 flex-wrap">
            <div className="relative max-w-md flex-1">
              <Input
                type="search"
                placeholder="搜索模块..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pr-20"
              />
              {searchQuery && (
                <div className="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1">
                  <span className="text-xs text-muted-foreground">{searchResults?.length || 0} 个</span>
                  <button
                    onClick={() => setSearchQuery('')}
                    className="text-muted-foreground hover:text-foreground transition-colors p-1"
                    title="清除搜索"
                  >
                    ✕
                  </button>
                </div>
              )}
            </div>

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
                onChange={(e) => setSortBy(e.target.value as typeof sortBy)}
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
              全部 ({stats.total}, {formatBytes(stats.totalSize)})
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
                {type} ({moduleTypeStats.get(type)?.count ?? 0}, {formatBytes(moduleTypeStats.get(type)?.size ?? 0)})
              </button>
            ))}
          </div>
        </div>

        {/* 树形视图 */}
        <Card className="overflow-visible">
          <CardHeader>
            <CardTitle>模块树</CardTitle>
            <CardDescription>{searchResults ? `${searchResults.length} 个搜索结果` : '文件树结构'}</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="h-150 overflow-y-auto overflow-x-hidden">
              {searchResults ? (
                // 搜索结果列表
                <div className="space-y-1">
                  {searchResults.map((node) => (
                    <div
                      key={node.fullPath}
                      className="flex items-center gap-2 py-1.5 px-2 hover:bg-muted/50 rounded text-sm"
                    >
                      <span className="shrink-0">📄</span>
                      <span className="flex-1 font-mono text-xs truncate" title={node.fullPath}>
                        {node.fullPath}
                      </span>
                      <Badge variant={getSizeBadgeVariant(node.size)} className={BADGE_STYLES.SIZE}>
                        {formatBytes(node.size)}
                      </Badge>
                      {node.module && (
                        <Badge
                          variant={node.module.isNodeModule ? 'secondary' : 'outline'}
                          className={BADGE_STYLES.COMPACT}
                        >
                          {node.module.isNodeModule ? 'npm' : 'src'}
                        </Badge>
                      )}
                    </div>
                  ))}
                </div>
              ) : (
                // 树形视图
                <>
                  {rootNodes.map((node) => (
                    <TreeNodeComponent
                      key={node.fullPath}
                      node={node}
                      level={0}
                      sortBy={sortBy}
                      sortDirection={sortDirection}
                      refCountMap={refCountMap}
                      allChunks={data.chunks}
                      highlightedModuleName={highlightedModuleName}
                      highlightedModulePath={highlightedModulePath}
                      expandedPaths={expandedPaths}
                    />
                  ))}
                </>
              )}
            </div>
          </CardContent>
        </Card>

        {searchResults && searchResults.length === 0 && (
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
