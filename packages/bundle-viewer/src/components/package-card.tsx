import { useMemo, useState } from 'react'
import type { StandardBundleData, Package, Module } from '@/types/bundle-data-standard'
import { type PackageGroup } from '@/lib/package-analyzer'
import { beautifyModulePath, getModuleName, getPathCategory } from '@/lib/utils/path-utils'
import { buildModuleTree, getCompressedPath, sortTreeNodesBySize, type TreeNode } from '@/lib/tree-builder'
import { Card, CardContent, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { formatBytes, getSizeBadgeVariant, formatNumber } from '@/lib/utils/format'
import { getFileExtension } from '@/lib/utils/file-utils'
import { BADGE_STYLES } from '@/lib/constants'
import { navigateTo } from '@/lib/navigation'
import { cn } from '@/lib/utils'

interface PackageCardProps {
  group: PackageGroup
  allPackages: Package[]
  allModules: Module[]
  data: StandardBundleData
}

interface ModuleTreeNodeProps {
  node: TreeNode
  level: number
  allModules: Module[]
  packageName?: string
}

export function PackageCard({ group, allPackages, allModules, data }: PackageCardProps) {
  const [isExpanded, setIsExpanded] = useState(false)
  const [expandedVersions, setExpandedVersions] = useState<Set<number>>(new Set())
  const [moduleViewMode, setModuleViewMode] = useState<'list' | 'tree'>('tree')

  // 直接使用标准结构的 moduleMap（按 ID 索引）
  const moduleMap = useMemo(() => data.moduleMap, [data.moduleMap])
  const isSingleVersion = group.versionCount === 1

  const toggleVersionModules = (idx: number) => {
    const newSet = new Set(expandedVersions)
    if (newSet.has(idx)) {
      newSet.delete(idx)
    } else {
      newSet.add(idx)
    }
    setExpandedVersions(newSet)
  }

  // 获取每个版本的详细包信息
  const versionDetails = useMemo(() => {
    return group.versions.map((version) => {
      const pkg = allPackages.find((p) => p.name === group.name && p.version === version.version)
      return { ...version, modules: pkg?.moduleIds || [] }
    })
  }, [group, allPackages])

  if (isSingleVersion) {
    return (
      <Collapsible open={expandedVersions.has(0)} onOpenChange={() => toggleVersionModules(0)}>
        <Card className="gap-0! py-0!">
          <CollapsibleTrigger className="hover:bg-muted/50 cursor-pointer text-sm px-4 py-2! text-muted-foreground hover:text-foreground transition-colors">
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <CardTitle className="text-base flex gap-3 flex-wrap items-center">
                  <span className="font-mono">{group.name}</span>
                  <div className="flex gap-1.5 flex-wrap items-center">
                    <Badge variant={getSizeBadgeVariant(group.totalSize)} className={BADGE_STYLES.SIZE}>
                      {formatBytes(group.totalSize)}
                    </Badge>
                    <Badge variant="outline" className={BADGE_STYLES.NUMERIC}>
                      {formatNumber(group.totalModuleCount)} 个模块
                    </Badge>
                    <Badge
                      variant={group.versionCount > 1 ? 'destructive' : 'outline'}
                      className={BADGE_STYLES.NUMERIC}
                    >
                      {group.versionCount} 个版本
                    </Badge>
                  </div>
                </CardTitle>
              </div>
            </div>
          </CollapsibleTrigger>
          <CardContent>
            {versionDetails[0]?.modules.length > 0 && (
              <CollapsibleContent className="max-h-60 overflow-y-auto flex flex-col gap-2 my-3">
                <div className="inline-flex items-center gap-2">
                  {expandedVersions.has(0) && (
                    <div className="flex gap-1">
                      <button
                        onClick={(e) => {
                          e.stopPropagation()
                          setModuleViewMode('tree')
                        }}
                        className={cn(
                          'px-2 py-0.5 text-xs rounded transition-colors',
                          moduleViewMode === 'tree'
                            ? 'bg-primary text-primary-foreground'
                            : 'bg-muted/50 hover:bg-muted',
                        )}
                      >
                        树形
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation()
                          setModuleViewMode('list')
                        }}
                        className={cn(
                          'px-2 py-0.5 text-xs rounded transition-colors',
                          moduleViewMode === 'list'
                            ? 'bg-primary text-primary-foreground'
                            : 'bg-muted/50 hover:bg-muted',
                        )}
                      >
                        列表
                      </button>
                    </div>
                  )}
                </div>
                {moduleViewMode === 'list' ? (
                  <div className="space-y-1">
                    {versionDetails[0].modules
                      .map((id) => moduleMap.get(id))
                      .filter((m): m is Module => m !== undefined)
                      .toSorted((a, b) => b.size - a.size)
                      .map((module, mIdx) => {
                        const [moduleName, fullPathWithLoader] = getModuleName(module.name, allModules)
                        const category = getPathCategory(moduleName)
                        const fileExt = getFileExtension(moduleName)
                        return (
                          <div
                            key={mIdx}
                            className="text-xs p-1.5 rounded bg-background/50 font-mono flex items-center gap-2 cursor-pointer hover:bg-muted/80 transition-colors"
                            onClick={() =>
                              navigateTo('modules', {
                                moduleName: module.name,
                                packageName: group.name,
                              })
                            }
                            title="点击查看模块详情"
                          >
                            <span
                              className={cn(
                                'flex-1 truncate',
                                category === 'project' ? 'text-foreground' : 'text-muted-foreground',
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
                              <Badge variant={getSizeBadgeVariant(module.size)} className={BADGE_STYLES.SIZE}>
                                {formatBytes(module.size)}
                              </Badge>
                            </div>
                          </div>
                        )
                      })}
                  </div>
                ) : (
                  (() => {
                    const modules = versionDetails[0].modules.map((id) => moduleMap.get(id)).filter(Boolean) as Module[]
                    const tree = buildModuleTree(modules)
                    const rootNodes = Array.from(tree.children.values())
                    return (
                      <div className="space-y-0.5 w-full max-w-full overflow-hidden">
                        {sortTreeNodesBySize(rootNodes).map((node) => (
                          <ModuleTreeNodeForPackage
                            key={node.fullPath}
                            node={node}
                            level={0}
                            allModules={allModules}
                            packageName={group.name}
                          />
                        ))}
                      </div>
                    )
                  })()
                )}
              </CollapsibleContent>
            )}
          </CardContent>
        </Card>
      </Collapsible>
    )
  }

  return (
    <Collapsible open={isExpanded} onOpenChange={setIsExpanded}>
      <Card className="gap-0! py-0!">
        <CollapsibleTrigger className="hover:bg-muted/50 cursor-pointer text-sm px-4 py-2! text-muted-foreground hover:text-foreground transition-colors">
          <div className="flex items-start justify-between">
            <div className="flex-1">
              <CardTitle className="text-base flex gap-3 flex-wrap items-center">
                <span className="font-mono">{group.name}</span>
                <div className="flex gap-1.5 flex-wrap items-center">
                  <Badge variant={getSizeBadgeVariant(group.totalSize)} className={BADGE_STYLES.SIZE}>
                    {formatBytes(group.totalSize)}
                  </Badge>
                  <Badge variant="outline" className={BADGE_STYLES.NUMERIC}>
                    {formatNumber(group.totalModuleCount)} 个模块
                  </Badge>
                  <Badge variant={group.versionCount > 1 ? 'destructive' : 'outline'} className={BADGE_STYLES.NUMERIC}>
                    {group.versionCount} 个版本
                  </Badge>
                </div>
              </CardTitle>
            </div>
          </div>
        </CollapsibleTrigger>
        <CardContent>
          <CollapsibleContent className="my-3 space-y-3">
            {versionDetails.map((version, idx) => (
              <div key={idx} className="rounded-md bg-muted/50 overflow-hidden">
                <div className="flex items-center justify-between p-2">
                  <div className="flex items-center gap-1.5 flex-1 flex-wrap h-6">
                    <Badge variant="default" className={BADGE_STYLES.NUMERIC}>
                      v{version.version}
                    </Badge>
                    <Badge variant={getSizeBadgeVariant(version.size)} className={BADGE_STYLES.SIZE}>
                      {formatBytes(version.size)}
                    </Badge>
                    <button
                      onClick={() => toggleVersionModules(idx)}
                      className="text-xs text-muted-foreground hover:text-foreground transition-colors flex items-center gap-1.5"
                    >
                      {expandedVersions.has(idx) ? '▼' : '▶'}
                      <Badge variant="outline" className={BADGE_STYLES.NUMERIC}>
                        {version.moduleCount} 个模块
                      </Badge>
                    </button>
                    {expandedVersions.has(idx) && (
                      <div className="flex gap-1 ml-2">
                        <button
                          onClick={(e) => {
                            e.stopPropagation()
                            setModuleViewMode('tree')
                          }}
                          className={cn(
                            'px-2 py-0.5 text-xs rounded transition-colors',
                            moduleViewMode === 'tree'
                              ? 'bg-primary text-primary-foreground'
                              : 'bg-muted/50 hover:bg-muted',
                          )}
                        >
                          树形
                        </button>
                        <button
                          onClick={(e) => {
                            e.stopPropagation()
                            setModuleViewMode('list')
                          }}
                          className={cn(
                            'px-2 py-0.5 text-xs rounded transition-colors',
                            moduleViewMode === 'list'
                              ? 'bg-primary text-primary-foreground'
                              : 'bg-muted/50 hover:bg-muted',
                          )}
                        >
                          列表
                        </button>
                      </div>
                    )}
                  </div>
                </div>

                {expandedVersions.has(idx) && version.modules.length > 0 && (
                  <div className="px-2 pb-2 max-h-60 overflow-y-auto">
                    {moduleViewMode === 'list' ? (
                      <div className="space-y-1">
                        {version.modules
                          .map((id) => moduleMap.get(id))
                          .filter((m): m is Module => m !== undefined)
                          .toSorted((a, b) => b.size - a.size)
                          .map((module, mIdx) => {
                            const [moduleName, fullPathWithLoader] = getModuleName(module.name, allModules)
                            const category = getPathCategory(moduleName)
                            const fileExt = getFileExtension(moduleName)
                            return (
                              <div
                                key={mIdx}
                                className="text-xs p-1.5 rounded bg-background/50 font-mono flex items-center gap-2 cursor-pointer hover:bg-muted/80 transition-colors"
                                onClick={() =>
                                  navigateTo('modules', {
                                    moduleName: module.name,
                                    packageName: group.name,
                                  })
                                }
                                title="点击查看模块详情"
                              >
                                <span
                                  className={cn(
                                    'flex-1 truncate',
                                    category === 'project' ? 'text-foreground' : 'text-muted-foreground',
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
                                  <Badge variant={getSizeBadgeVariant(module.size)} className={BADGE_STYLES.SIZE}>
                                    {formatBytes(module.size)}
                                  </Badge>
                                </div>
                              </div>
                            )
                          })}
                      </div>
                    ) : (
                      (() => {
                        const modules = version.modules.map((id) => moduleMap.get(id)).filter(Boolean) as Module[]
                        const tree = buildModuleTree(modules)
                        const rootNodes = Array.from(tree.children.values())
                        return (
                          <div className="space-y-0.5 w-full max-w-full overflow-hidden">
                            {sortTreeNodesBySize(rootNodes).map((node) => (
                              <ModuleTreeNodeForPackage
                                key={node.fullPath}
                                node={node}
                                level={0}
                                allModules={allModules}
                                packageName={group.name}
                              />
                            ))}
                          </div>
                        )
                      })()
                    )}
                  </div>
                )}
              </div>
            ))}
          </CollapsibleContent>
        </CardContent>
      </Card>
    </Collapsible>
  )
}

function ModuleTreeNodeForPackage({ node, level, allModules, packageName }: ModuleTreeNodeProps) {
  const [isExpanded, setIsExpanded] = useState(false)

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
    return sortTreeNodesBySize(children)
  }, [targetNode])

  const hasChildren = childNodes.length > 0

  const handleToggle = (e: React.MouseEvent) => {
    if (hasChildren && !node.isFile) {
      e.stopPropagation()
      setIsExpanded(!isExpanded)
    }
  }

  const handleClick = (e: React.MouseEvent) => {
    if (node.isFile && node.module) {
      e.stopPropagation()
      navigateTo('modules', { moduleName: node.module.name, packageName })
    }
  }

  const getIcon = () => {
    if (node.isFile) return '📄'
    return isExpanded ? '📂' : '📁'
  }

  const [moduleName] = node.module ? getModuleName(node.module.name, allModules) : ['']
  const category = node.module ? getPathCategory(moduleName) : 'project'
  const fileExt = node.isFile ? getFileExtension(moduleName) : null

  return (
    <div>
      <div
        className={cn(
          'flex items-center gap-2 py-0.5 px-1 rounded text-xs',
          node.isFile ? 'hover:bg-muted/80 cursor-pointer' : 'hover:bg-muted/30 cursor-pointer font-medium',
        )}
        style={{ paddingLeft: `${level * 12}px` }}
        onClick={node.isFile ? handleClick : handleToggle}
        title={node.isFile ? '点击查看模块详情' : undefined}
      >
        <span className="shrink-0 select-none text-xs" onClick={hasChildren && !node.isFile ? handleToggle : undefined}>
          {hasChildren ? (isExpanded ? '▼' : '▶') : ' '}
        </span>
        <span className="shrink-0 text-xs">{getIcon()}</span>
        <span
          className={cn(
            'flex-1 truncate font-mono',
            category === 'project' ? 'text-foreground' : 'text-muted-foreground',
          )}
          title={displayName}
        >
          {displayName}
        </span>
        <div className="flex gap-1.5 shrink-0 items-center">
          {/* 文件夹显示子项统计 */}
          {!node.isFile && childStats && (
            <span className="text-muted-foreground text-[0.65rem] whitespace-nowrap font-mono">
              ({childStats.fileCount} 个文件)
            </span>
          )}
          {node.isFile && fileExt && (
            <Badge variant="outline" className={BADGE_STYLES.COMPACT}>
              .{fileExt}
            </Badge>
          )}
          {node.isFile && node.module?.isNodeModule && (
            <Badge variant="secondary" className={BADGE_STYLES.COMPACT}>
              pkg
            </Badge>
          )}
          <Badge
            variant={getSizeBadgeVariant(node.size)}
            className={cn(BADGE_STYLES.SIZE, !node.isFile && 'opacity-90')}
            title={node.isFile ? `文件体积: ${formatBytes(node.size)}` : `文件夹总计: ${formatBytes(node.size)}`}
          >
            {formatBytes(node.size)}
          </Badge>
        </div>
      </div>

      {isExpanded && hasChildren && (
        <div>
          {childNodes.map((child) => (
            <ModuleTreeNodeForPackage
              key={child.fullPath}
              node={child}
              level={level + 1}
              allModules={allModules}
              packageName={packageName}
            />
          ))}
        </div>
      )}
    </div>
  )
}
