/**
 * 路径标准化序列化器
 * 将绝对路径替换为 <ROOT> 占位符，确保快照的可移植性
 */

import type { SnapshotSerializer } from 'vitest';

function normalizePathsInValue(val: any, cwd: string): any {
  if (typeof val === 'string') {
    return val.replaceAll(cwd, '<ROOT>');
  }
  
  if (Array.isArray(val)) {
    return val.map(item => normalizePathsInValue(item, cwd));
  }
  
  if (val && typeof val === 'object' && val.constructor === Object) {
    const normalized: any = {};
    for (const [key, value] of Object.entries(val)) {
      normalized[key] = normalizePathsInValue(value, cwd);
    }
    return normalized;
  }
  
  return val;
}

function containsPaths(val: unknown, cwd: string): boolean {
  if (typeof val === 'string') {
    return val.includes(cwd);
  }
  
  if (Array.isArray(val)) {
    return val.some(item => containsPaths(item, cwd));
  }
  
  if (val && typeof val === 'object' && val.constructor === Object) {
    return Object.values(val).some(value => containsPaths(value, cwd));
  }
  
  return false;
}

export const pathNormalizerSerializer: SnapshotSerializer = {
  test: (val: unknown): boolean => 
    containsPaths(val, process.cwd()),
  
  serialize: (val: any) => {
    const normalized = normalizePathsInValue(val, process.cwd());
    return JSON.stringify(normalized, null, 2);
  },
};