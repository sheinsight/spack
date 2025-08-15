/**
 * 路径标准化序列化器
 * 将绝对路径替换为 <ROOT> 占位符，确保快照的可移植性
 * 支持跨平台路径格式（Unix 和 Windows）
 */

import type { SnapshotSerializer } from 'vitest';
import * as path from 'node:path';

// 获取当前工作目录的多种格式
const cwd = process.cwd();
const cwdPosix = cwd.replace(/\\/g, '/'); // Unix 风格路径
const cwdWin32 = cwd.replace(/\//g, '\\'); // Windows 风格路径

// Windows 驱动器路径正则表达式 (C:\, D:\, etc.)
const WINDOWS_DRIVE_REGEX = /^[A-Za-z]:\\/;

// 绝对路径检测正则表达式
const ABSOLUTE_PATH_REGEXES = [
  /^\/[^\/]/,              // Unix 绝对路径: /path
  /^[A-Za-z]:\\/,          // Windows 绝对路径: C:\path
  /^\\\\[^\\]/,            // UNC 路径: \\server\path
];

function isAbsolutePath(str: string): boolean {
  return ABSOLUTE_PATH_REGEXES.some(regex => regex.test(str));
}

function normalizePathString(str: string): string {
  // 先尝试替换当前工作目录（Unix 格式）
  if (str.includes(cwdPosix)) {
    str = str.replaceAll(cwdPosix, '<ROOT>');
  }
  
  // 再尝试替换当前工作目录（Windows 格式）
  if (str.includes(cwdWin32)) {
    str = str.replaceAll(cwdWin32, '<ROOT>');
  }
  
  // 对于其他绝对路径，也进行标准化
  // Windows 路径标准化
  if (WINDOWS_DRIVE_REGEX.test(str)) {
    str = str.replace(WINDOWS_DRIVE_REGEX, '<DRIVE>/');
  }
  
  // 将所有反斜杠转换为正斜杠以保持一致性
  str = str.replace(/\\/g, '/');
  
  return str;
}

function normalizePathsInValue(val: any): any {
  if (typeof val === 'string') {
    return normalizePathString(val);
  }
  
  if (Array.isArray(val)) {
    return val.map(item => normalizePathsInValue(item));
  }
  
  if (val && typeof val === 'object' && val.constructor === Object) {
    const normalized: any = {};
    for (const [key, value] of Object.entries(val)) {
      normalized[key] = normalizePathsInValue(value);
    }
    return normalized;
  }
  
  return val;
}

function containsAbsolutePaths(val: unknown): boolean {
  if (typeof val === 'string') {
    // 检查是否包含当前工作目录或任何绝对路径
    return val.includes(cwdPosix) || 
           val.includes(cwdWin32) || 
           isAbsolutePath(val);
  }
  
  if (Array.isArray(val)) {
    return val.some(item => containsAbsolutePaths(item));
  }
  
  if (val && typeof val === 'object' && val.constructor === Object) {
    return Object.values(val).some(value => containsAbsolutePaths(value));
  }
  
  return false;
}

export const pathNormalizerSerializer: SnapshotSerializer = {
  test: (val: unknown): boolean => 
    containsAbsolutePaths(val),
  
  serialize: (val: any) => {
    const normalized = normalizePathsInValue(val);
    return JSON.stringify(normalized, null, 2);
  },
};