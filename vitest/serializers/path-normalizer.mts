/**
 * 路径标准化序列化器
 * 将绝对路径替换为 <ROOT> 占位符，确保快照的可移植性
 */

import type { SnapshotSerializer } from 'vitest';

export const pathNormalizerSerializer: SnapshotSerializer = {
  test: (val: unknown): val is string => 
    typeof val === 'string' && val.includes(process.cwd()),
  
  serialize: (val: string) => {
    return `"${val.replaceAll(process.cwd(), '<ROOT>')}"`;
  },
};