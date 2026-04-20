import { useEffect, useState, useMemo, useRef } from 'react'
import { Badge } from '@/components/ui/badge'
import {
  buildModuleTree,
  sortTreeNodes,
  sortTreeNodesBySize,
  getCompressedPath,
  type TreeNode,
} from '@/lib/tree-builder'
import { formatBytes, getSizeBadgeVariant } from '@/lib/utils/format'
import { BADGE_STYLES } from '@/lib/constants'
import { navigateTo } from '@/lib/navigation'
import { getPathCategory, getModuleName } from '@/lib/utils/path-utils'
import { getFileExtension } from '@/lib/utils/file-utils'
import type { StandardBundleData, Module, Chunk } from '@/types/bundle-data-standard'
import { cn } from '@/lib/utils'

interface ChunkTreeNodeProps {
  chunk: Chunk
  data: StandardBundleData
  level: number
  sortBy: 'name' | 'size'
  isHighlighted?: boolean
}

export function ChunkTreeNode({ chunk, data, level, sortBy, isHighlighted }: ChunkTreeNodeProps) {
  const [isExpanded, setIsExpanded] = useState(isHighlighted || false)
  const [modulesExpanded, setModulesExpanded] = useState(false)

  // 标记是否正在手动收起（避免竞态条件）
  const isManuallyCollapsingRef = useRef(false)

  // 响应高亮状态变化，自动展开（但不阻止手动收起）
  useEffect(() => {
    if (isHighlighted && !isExpanded && !isManuallyCollapsingRef.current) {
      setIsExpanded(true)
    }

    // 当高亮清除时，重置手动收起标记
    if (!isHighlighted) {
      isManuallyCollapsingRef.current = false
    }
  }, [isHighlighted, isExpanded])

  // 直接使用标准结构的 Map
  const moduleMap = data.moduleMap

  const modules = useMemo(() => {
    return chunk.moduleIds.map((moduleId) => moduleMap.get(moduleId)).filter(Boolean) as Module[]
  }, [chunk.moduleIds, moduleMap])

  const moduleTree = useMemo(() => buildModuleTree(modules), [modules])

  const rootNodes = useMemo(() => {
    const children = Array.from(moduleTree.children.values())
    return sortBy === 'size' ? sortTreeNodesBySize(children) : sortTreeNodes(children)
  }, [moduleTree, sortBy])

  const thirdPartyModules = modules.filter((m) => m.isNodeModule)
  const projectModules = modules.filter((m) => !m.isNodeModule)
  const thirdPartySize = thirdPartyModules.reduce((sum, m) => sum + m.size, 0)
  const projectSize = projectModules.reduce((sum, m) => sum + m.size, 0)

  const getChunkType = () => {
    const types = []
    if (chunk.type === 'entry') types.push('入口')
    if (chunk.type === 'initial' || chunk.type === 'entry') types.push('初始')
    if (chunk.type === 'runtime') types.push('运行时')
    if (chunk.type === 'async') types.push('异步')
    return types
  }

  return (
    <>
      <div
        id={`chunk-${chunk.id}`}
        className={cn(
          'flex items-center gap-2 py-1.5 px-2 hover:bg-muted/50 rounded cursor-pointer group font-medium text-sm bg-background/95 backdrop-blur-sm z-20 border-b border-border/50',
          isHighlighted && 'ring-2 ring-primary shadow-lg'
        )}
        style={{
          paddingLeft: `${level * 16 + 8}px`,
          position: 'sticky',
          top: `${level * 40}px`,
        }}
        onClick={() => {
          // 如果正在收起且是高亮的，清除高亮
          if (isExpanded && isHighlighted) {
            isManuallyCollapsingRef.current = true
            navigateTo('chunks') // 清除 chunkId
          }
          setIsExpanded(!isExpanded)
        }}
      >
        <span className="shrink-0 select-none">{isExpanded ? '▼' : '▶'}</span>
        <span className="flex-1 font-mono text-xs truncate" title={chunk.names.join(', ') || chunk.id}>
          {chunk.names.join(', ') || chunk.id}
        </span>
        <div className="flex gap-1.5 shrink-0 items-center">
          {/* 代码分布进度条 - 固定宽度 */}
          <div
            className="w-32 h-2 rounded-full overflow-hidden relative bg-muted/30"
            title={`第三方: ${chunk.size > 0 ? ((thirdPartySize / chunk.size) * 100).toFixed(1) : 0}% | 项目: ${chunk.size > 0 ? ((projectSize / chunk.size) * 100).toFixed(1) : 0}%`}
          >
            {/* 第三方依赖 - 碧绿色 (bg-data-viz-2) */}
            <div
              className="absolute top-0 left-0 h-full bg-data-viz-2 transition-all"
              style={{
                width: `${chunk.size > 0 ? (thirdPartySize / chunk.size) * 100 : 0}%`,
              }}
            />
            {/* 项目源码 - 钴蓝色 (bg-data-viz-1) */}
            <div
              className="absolute top-0 h-full bg-data-viz-1 transition-all"
              style={{
                left: `${chunk.size > 0 ? (thirdPartySize / chunk.size) * 100 : 0}%`,
                width: `${chunk.size > 0 ? (projectSize / chunk.size) * 100 : 0}%`,
              }}
            />
          </div>
          <Badge variant={getSizeBadgeVariant(chunk.size)} className={BADGE_STYLES.SIZE}>
            {formatBytes(chunk.size)}
          </Badge>
          <Badge variant="outline" className={BADGE_STYLES.NUMERIC}>
            {modules.length}
          </Badge>
          {getChunkType().map((type) => (
            <Badge
              key={type}
              variant={type === '入口' || type === '初始' ? 'default' : 'outline'}
              className={BADGE_STYLES.COMPACT}
            >
              {type}
            </Badge>
          ))}
        </div>
      </div>

      {isExpanded && (
        <div
          className="py-2 px-2 border-l-2 border-muted ml-4 space-y-2"
          style={{ paddingLeft: `${level * 16 + 24}px` }}
        >
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
                style={{
                  width: `${chunk.size > 0 ? (thirdPartySize / chunk.size) * 100 : 0}%`,
                }}
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
              <p className="text-xs text-muted-foreground mb-1">输出文件：</p>
              <div className="space-y-1">
                {(chunk.ext?.files || []).map((file) => (
                  <p key={file} className="text-xs font-mono bg-muted px-2 py-1 rounded">
                    {file}
                  </p>
                ))}
              </div>
            </div>
          )}

          {/* 模块树 */}
          {rootNodes.length > 0 && (
            <div>
              <button
                onClick={() => setModulesExpanded(!modulesExpanded)}
                className="text-xs text-muted-foreground hover:text-foreground transition-colors"
              >
                {modulesExpanded ? '▼' : '▶'} 查看 {modules.length} 个模块（树形视图）
              </button>

              {modulesExpanded && (
                <div className="mt-2 border-l-2 border-muted/50 pl-2">
                  {rootNodes.map((node: TreeNode) => (
                    <ModuleTreeNodeForChunk
                      key={node.fullPath}
                      node={node}
                      modules={data.modules}
                      level={0}
                      sortBy={sortBy}
                    />
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      )}
    </>
  )
}

interface ModuleTreeNodeProps {
  node: TreeNode
  modules: Module[]
  level: number
  sortBy: 'name' | 'size'
}

export function ModuleTreeNodeForChunk({ node, modules, level, sortBy }: ModuleTreeNodeProps) {
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
    return sortBy === 'size' ? sortTreeNodesBySize(children) : sortTreeNodes(children)
  }, [targetNode, sortBy])

  const hasChildren = childNodes.length > 0

  const handleToggle = () => {
    // 文件节点：跳转到模块详情
    if (node.isFile && node.module) {
      navigateTo('modules', { moduleName: node.module.name })
      return
    }

    // 文件夹节点：展开/收起
    if (hasChildren) {
      setIsExpanded(!isExpanded)
    }
  }

  const getIcon = () => {
    if (node.isFile) return '📄'
    return isExpanded ? '📂' : '📁'
  }

  const category = node.module ? getPathCategory(getModuleName(node.module.name, modules)[0]) : 'project'
  const fileExt = node.isFile ? getFileExtension(node.name) : null

  return (
    <>
      <div
        className={cn(
          'flex items-center gap-2 py-0.5 px-1 hover:bg-muted/30 rounded cursor-pointer text-xs',
          !node.isFile && 'font-medium bg-background/95 backdrop-blur-sm z-10 border-b border-border/50'
        )}
        style={{
          paddingLeft: `${level * 12}px`,
          position: node.isFile ? undefined : 'sticky',
          // 基础偏移 34px（ChunkTreeNode 高度）+ level * 21px（ModuleTreeNode 高度）
          top: node.isFile ? undefined : `${34 + level * 21}px`,
          zIndex: node.isFile ? undefined : 100 - level, // 层级越浅，z-index 越高
        }}
        onClick={handleToggle}
      >
        <span className="shrink-0 select-none text-xs">{hasChildren ? (isExpanded ? '▼' : '▶') : ' '}</span>
        <span className="shrink-0 text-xs">{getIcon()}</span>
        <span
          className={cn(
            'flex-1 truncate font-mono',
            category === 'project' ? 'text-foreground' : 'text-muted-foreground'
          )}
          title={displayName}
        >
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
          {childNodes.map((child: TreeNode) => (
            <ModuleTreeNodeForChunk
              key={child.fullPath}
              node={child}
              modules={modules}
              level={level + 1}
              sortBy={sortBy}
            />
          ))}
        </div>
      )}
    </>
  )
}
