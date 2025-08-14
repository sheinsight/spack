/**
 * 通用数组序列化器
 * 处理所有数组类型，使用深度排序确保一致性
 */

import type { SnapshotSerializer } from 'vitest';
import { deepSort } from './deep-sort.mts';

export const generalArraySerializer: SnapshotSerializer = {
  test: (val: unknown): val is Array<any> => {
    return Array.isArray(val);
  },
  
  serialize: (val: any[]) => {
    // 使用深度排序处理嵌套结构
    const sorted = deepSort(val);
    
    return JSON.stringify(sorted, null, 2);
  }
};