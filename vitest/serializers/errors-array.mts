/**
 * 错误数组序列化器
 * 专门处理包含 moduleId 或 message 属性的错误对象数组
 */

import type { SnapshotSerializer } from 'vitest';
import { deepSort } from './deep-sort.mts';

export const errorsArraySerializer: SnapshotSerializer = {
  test: (val: unknown): val is Array<any> => {
    return Array.isArray(val) && 
           val.some(item => typeof item === 'object' && 
                           item !== null && 
                           ('moduleId' in item || 'message' in item));
  },
  
  serialize: (val: any[]) => {
    // 先进行深度排序，然后按 moduleId/message 重新排序顶层数组
    const deepSorted = deepSort(val);
    const finalSorted = deepSorted.sort((a: any, b: any) => {
      // 处理 null/undefined 值
      if (a === null || a === undefined) {
        if (b === null || b === undefined) return 0;
        return 1; // null/undefined 排在后面
      }
      if (b === null || b === undefined) {
        return -1; // 非空值排在前面
      }
      
      const keyA = a.moduleId || a.message || '';
      const keyB = b.moduleId || b.message || '';
      return keyA.localeCompare(keyB);
    });
    
    return JSON.stringify(finalSorted, null, 2);
  }
};