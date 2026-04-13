import type { Module } from '@/types/bundle-data-standard'

export interface TreeNode {
  name: string // 文件/文件夹名
  fullPath: string // 完整路径
  isFile: boolean
  size: number
  module?: Module
  moduleName?: string // 模块名称，用于快速匹配
  children: Map<string, TreeNode>
  depth: number
}

/**
 * 构建模块路径树
 */
export function buildModuleTree(modules: Module[]): TreeNode {
  const root: TreeNode = {
    name: '',
    fullPath: '',
    isFile: false,
    size: 0,
    children: new Map(),
    depth: 0,
  }

  for (const module of modules) {
    // 使用 module.name 作为路径
    let path = module.name

    // 处理特殊路径前缀
    if (path.startsWith('asset|')) {
      path = path.substring(6)
    }

    if (path.startsWith('ignored|')) {
      path = path.substring(8)
    }

    // 处理 data: 协议路径，提取实际路径部分
    if (path.startsWith('data:')) {
      root.children.set(path, {
        name: path,
        fullPath: path,
        isFile: true,
        size: module.size,
        module,
        moduleName: module.name,
        children: new Map(),
        depth: 1,
      })

      continue
    }

    // 分割路径
    const parts = path.split(/[/\\]/).filter(Boolean)

    if (parts.length === 0) {
      continue
    }

    let current = root

    // 构建树
    for (let i = 0; i < parts.length; i++) {
      const part = parts[i]
      const isLastPart = i === parts.length - 1
      const currentPath = parts.slice(0, i + 1).join('/')

      if (!current.children.has(part)) {
        current.children.set(part, {
          name: part,
          fullPath: currentPath,
          isFile: isLastPart,
          size: isLastPart ? module.size : 0,
          module: isLastPart ? module : undefined,
          moduleName: isLastPart ? module.name : undefined,
          children: new Map(),
          depth: i + 1,
        })
      } else if (isLastPart) {
        // 更新文件信息（同一路径可能对应多个模块，如 Concatenated 模块）
        const node = current.children.get(part)!
        node.size += module.size
        node.module = module
        node.moduleName = module.name
      }

      current = current.children.get(part)!

      // 累加文件夹大小
      if (!isLastPart) {
        current.size += module.size
      }
    }
  }

  // 计算所有文件夹的总大小
  calculateFolderSizes(root)

  return root
}

/**
 * 递归计算文件夹大小
 */
function calculateFolderSizes(node: TreeNode): number {
  if (node.isFile) {
    return node.size
  }

  let totalSize = 0
  for (const child of node.children.values()) {
    totalSize += calculateFolderSizes(child)
  }

  node.size = totalSize
  return totalSize
}

/**
 * 获取压缩路径（类似 VSCode 的文件夹折叠）
 * 如果一个文件夹只有一个子文件夹，则合并显示
 */
export function getCompressedPath(node: TreeNode): { displayName: string; targetNode: TreeNode } {
  if (node.isFile || node.children.size !== 1) {
    return { displayName: node.name, targetNode: node }
  }

  const [childName, childNode] = Array.from(node.children.entries())[0]

  // 如果子节点也是文件夹且只有一个子节点，继续压缩
  if (!childNode.isFile && childNode.children.size === 1) {
    const childResult = getCompressedPath(childNode)
    return {
      displayName: `${node.name}/${childResult.displayName}`,
      targetNode: childResult.targetNode,
    }
  }

  // 如果子节点是文件，不压缩（显示文件夹）
  if (childNode.isFile) {
    return { displayName: node.name, targetNode: node }
  }

  // 子节点是有多个子项的文件夹，压缩一层
  return {
    displayName: `${node.name}/${childName}`,
    targetNode: childNode,
  }
}

/**
 * 排序树节点：文件夹在前，按名称排序
 */
export function sortTreeNodes(nodes: TreeNode[]): TreeNode[] {
  return nodes.toSorted((a, b) => {
    // 文件夹在前
    if (a.isFile !== b.isFile) {
      return a.isFile ? 1 : -1
    }
    // 按名称排序
    return a.name.localeCompare(b.name)
  })
}

/**
 * 按大小排序树节点
 */
export function sortTreeNodesBySize(nodes: TreeNode[]): TreeNode[] {
  return nodes.toSorted((a, b) => {
    // 文件夹在前
    if (a.isFile !== b.isFile) {
      return a.isFile ? 1 : -1
    }
    // 按大小降序
    return b.size - a.size
  })
}

/**
 * 搜索树节点
 */
export function searchTree(node: TreeNode, query: string): TreeNode[] {
  const results: TreeNode[] = []
  const lowerQuery = query.toLowerCase()

  function search(n: TreeNode) {
    if (n.isFile && n.fullPath.toLowerCase().includes(lowerQuery)) {
      results.push(n)
    }

    for (const child of n.children.values()) {
      search(child)
    }
  }

  search(node)
  return results
}
