/**
 * 重复依赖检测结果序列化器
 * 专门处理包含 groups 数组的重复依赖检测结果
 */

import type { SnapshotSerializer } from 'vitest';
import { deepSort } from './deep-sort.mts';

export const duplicateDependencySerializer: SnapshotSerializer = {
  test: (val: unknown): val is object => {
    return typeof val === 'object' && 
           val !== null && 
           'groups' in val && 
           Array.isArray((val as any).groups);
  },
  
  serialize: (val: any) => {
    // 深拷贝并使用深度排序
    const copied = JSON.parse(JSON.stringify(val));
    const sorted = deepSort(copied);
    
    return JSON.stringify(sorted, null, 2);
  }
};