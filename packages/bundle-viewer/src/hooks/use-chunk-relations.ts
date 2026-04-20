/**
 * Chunk 依赖关系分析 Hooks
 */

import { useMemo } from 'react'
import type { StandardBundleData } from '@/types/bundle-data-standard'

export interface ChunkNode {
  id: string | number
  name: string
  size: number
  type: 'entry' | 'initial' | 'async' | 'runtime'
  parents: Array<string | number>
  children: Array<string | number>
  depth: number
  moduleCount: number
}

/**
 * 构建 Chunk 节点列表
 */
export function useChunkNodes(data: StandardBundleData) {
  return useMemo((): ChunkNode[] => {
    return data.chunks.map((chunk) => ({
      id: chunk.id,
      name: chunk.names[0] || `#${chunk.id}`,
      size: chunk.size,
      type: chunk.type,
      parents: chunk.parentIds,
      children: chunk.childIds,
      depth: 0,
      moduleCount: chunk.moduleIds.length,
    }))
  }, [data.chunks])
}

/**
 * 计算深度和检测循环依赖
 */
export function useChunkDepthAndCircular(chunkNodes: ChunkNode[]) {
  return useMemo(() => {
    const nodeMap = new Map(chunkNodes.map((n) => [n.id, { ...n }]))
    const visited = new Set<string | number>()
    const circles: Array<Array<string | number>> = []

    // BFS 计算深度
    const entryNodes = chunkNodes.filter((n) => n.type === 'entry')
    const queue: Array<{ id: string | number; depth: number; path: Array<string | number> }> = entryNodes.map((n) => ({
      id: n.id,
      depth: 0,
      path: [n.id],
    }))

    while (queue.length > 0) {
      const { id, depth, path } = queue.shift()!
      const node = nodeMap.get(id)
      if (!node) continue

      if (visited.has(id)) {
        // 检测循环
        const cycleStart = path.indexOf(id)
        if (cycleStart !== -1) {
          circles.push(path.slice(cycleStart))
        }
        continue
      }

      visited.add(id)
      node.depth = Math.max(node.depth, depth)

      for (const childId of node.children) {
        queue.push({ id: childId, depth: depth + 1, path: [...path, childId] })
      }
    }

    return {
      nodesWithDepth: Array.from(nodeMap.values()).toSorted((a, b) => a.depth - b.depth),
      circularPaths: circles,
    }
  }, [chunkNodes])
}

/**
 * 计算统计信息
 */
export function useChunkStats(nodesWithDepth: ChunkNode[], circularPaths: Array<Array<string | number>>) {
  return useMemo(() => {
    const entryCount = nodesWithDepth.filter((n) => n.type === 'entry').length
    const asyncCount = nodesWithDepth.filter((n) => n.type === 'async').length
    const maxDepth = Math.max(...nodesWithDepth.map((n) => n.depth), 0)
    const avgChildren = nodesWithDepth.reduce((sum, n) => sum + n.children.length, 0) / (nodesWithDepth.length || 1)

    return {
      entryCount,
      asyncCount,
      maxDepth,
      avgChildren,
      hasCircular: circularPaths.length > 0,
    }
  }, [nodesWithDepth, circularPaths])
}

/**
 * 获取节点颜色（基于类型）
 */
export function getNodeColor(type: ChunkNode['type']) {
  switch (type) {
    case 'entry':
      return 'bg-primary' // 最高优先级
    case 'runtime':
      return 'bg-primary opacity-80' // 高优先级
    case 'initial':
      return 'bg-primary opacity-60' // 中优先级
    case 'async':
      return 'bg-primary opacity-40' // 低优先级
    default:
      return 'bg-muted-foreground opacity-30'
  }
}

/**
 * 获取节点文字颜色
 */
export function getNodeTextColor(type: ChunkNode['type']) {
  switch (type) {
    case 'entry':
      return 'text-primary'
    case 'runtime':
      return 'text-primary opacity-90'
    case 'initial':
      return 'text-primary opacity-80'
    case 'async':
      return 'text-primary opacity-70'
    default:
      return 'text-muted-foreground'
  }
}
