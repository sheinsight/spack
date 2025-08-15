/**
 * 路径标准化序列化器单元测试
 */

import { describe, test, expect, beforeEach } from 'vitest';
import { pathNormalizerSerializer } from '../path-normalizer.mts';

describe('pathNormalizerSerializer', () => {
  const originalCwd = process.cwd();
  
  describe('test() 方法 - 路径检测', () => {
    test('应该检测 Unix 绝对路径', () => {
      expect(pathNormalizerSerializer.test('/usr/local/bin')).toBe(true);
      expect(pathNormalizerSerializer.test('/Users/developer/project')).toBe(true);
      expect(pathNormalizerSerializer.test('/tmp/file.txt')).toBe(true);
    });

    test('应该检测 Windows 绝对路径', () => {
      expect(pathNormalizerSerializer.test('C:\\Windows\\System32')).toBe(true);
      expect(pathNormalizerSerializer.test('D:\\Projects\\app')).toBe(true);
      expect(pathNormalizerSerializer.test('Z:\\network\\drive')).toBe(true);
    });

    test('应该检测 UNC 网络路径', () => {
      expect(pathNormalizerSerializer.test('\\\\server\\share\\folder')).toBe(true);
      expect(pathNormalizerSerializer.test('\\\\192.168.1.100\\shared')).toBe(true);
    });

    test('应该检测包含当前工作目录的路径', () => {
      const cwdPath = originalCwd + '/src/index.ts';
      const cwdPathWin = originalCwd.replace(/\//g, '\\') + '\\src\\index.ts';
      
      expect(pathNormalizerSerializer.test(cwdPath)).toBe(true);
      expect(pathNormalizerSerializer.test(cwdPathWin)).toBe(true);
    });

    test('应该忽略相对路径', () => {
      expect(pathNormalizerSerializer.test('./src/index.ts')).toBe(false);
      expect(pathNormalizerSerializer.test('../config/app.js')).toBe(false);
      expect(pathNormalizerSerializer.test('src/components/Button.tsx')).toBe(false);
    });

    test('应该检测嵌套对象中的绝对路径', () => {
      const objWithPaths = {
        message: 'Error in C:\\Projects\\app\\src\\index.ts',
        moduleId: '/usr/src/app/index.js',
        relative: './components/Button'
      };
      
      expect(pathNormalizerSerializer.test(objWithPaths)).toBe(true);
    });

    test('应该检测数组中的绝对路径', () => {
      const arrayWithPaths = [
        './relative/path',
        'C:\\Windows\\System32\\file.dll',
        'another/relative'
      ];
      
      expect(pathNormalizerSerializer.test(arrayWithPaths)).toBe(true);
    });

    test('应该忽略没有绝对路径的对象', () => {
      const objWithoutPaths = {
        message: 'Simple error message',
        code: 'ENOENT',
        relative: './src/index.ts'
      };
      
      expect(pathNormalizerSerializer.test(objWithoutPaths)).toBe(false);
    });
  });

  describe('serialize() 方法 - 路径标准化', () => {
    test('应该将当前工作目录替换为 <ROOT>', () => {
      const unixPath = originalCwd + '/src/index.ts';
      const windowsPath = originalCwd.replace(/\//g, '\\') + '\\src\\index.ts';
      
      const result1 = JSON.parse(pathNormalizerSerializer.serialize(unixPath));
      const result2 = JSON.parse(pathNormalizerSerializer.serialize(windowsPath));
      
      expect(result1).toBe('<ROOT>/src/index.ts');
      expect(result2).toBe('<ROOT>/src/index.ts');
    });

    test('应该将 Windows 驱动器路径替换为 <DRIVE>', () => {
      const paths = [
        'C:\\Windows\\System32\\notepad.exe',
        'D:\\Projects\\app\\src\\index.ts',
        'Z:\\network\\shared\\file.txt'
      ];
      
      paths.forEach(path => {
        const result = JSON.parse(pathNormalizerSerializer.serialize(path));
        expect(result).toMatch(/^<DRIVE>\//);
        expect(result).not.toContain('\\');
      });
    });

    test('应该将反斜杠转换为正斜杠', () => {
      const windowsPath = 'C:\\Users\\Developer\\Project\\file.js';
      const result = JSON.parse(pathNormalizerSerializer.serialize(windowsPath));
      
      expect(result).toBe('<DRIVE>/Users/Developer/Project/file.js');
      expect(result).not.toContain('\\');
    });

    test('应该处理 UNC 网络路径', () => {
      const uncPath = '\\\\server\\share\\folder\\file.txt';
      const result = JSON.parse(pathNormalizerSerializer.serialize(uncPath));
      
      expect(result).toBe('//server/share/folder/file.txt');
    });

    test('应该递归处理嵌套对象', () => {
      const complexObject = {
        message: `Error in "${originalCwd}/src/index.ts"`,
        moduleIdentifier: 'C:\\Projects\\app\\src\\index.ts',
        paths: [
          originalCwd + '/dist/bundle.js',
          'D:\\Temp\\cache.json',
          './relative/path'
        ],
        nested: {
          windowsPath: 'E:\\Data\\project\\file.txt',
          unixPath: '/opt/app/config.json',
          current: originalCwd.replace(/\//g, '\\') + '\\build\\output.js'
        }
      };
      
      const result = JSON.parse(pathNormalizerSerializer.serialize(complexObject));
      
      expect(result.message).toContain('<ROOT>/src/index.ts');
      expect(result.moduleIdentifier).toBe('<DRIVE>/Projects/app/src/index.ts');
      expect(result.paths[0]).toBe('<ROOT>/dist/bundle.js');
      expect(result.paths[1]).toBe('<DRIVE>/Temp/cache.json');
      expect(result.paths[2]).toBe('./relative/path'); // 相对路径保持不变
      expect(result.nested.windowsPath).toBe('<DRIVE>/Data/project/file.txt');
      expect(result.nested.current).toBe('<ROOT>/build/output.js');
    });

    test('应该处理数组中的路径', () => {
      const pathArray = [
        './relative/file.js',
        originalCwd + '/src/component.ts',
        'C:\\Windows\\System32\\kernel32.dll',
        '/usr/lib/libssl.so'
      ];
      
      const result = JSON.parse(pathNormalizerSerializer.serialize(pathArray));
      
      expect(result[0]).toBe('./relative/file.js');
      expect(result[1]).toBe('<ROOT>/src/component.ts');
      expect(result[2]).toBe('<DRIVE>/Windows/System32/kernel32.dll');
      expect(result[3]).toBe('/usr/lib/libssl.so');
    });
  });

  describe('边缘情况', () => {
    test('应该处理空字符串', () => {
      expect(pathNormalizerSerializer.test('')).toBe(false);
    });

    test('应该处理 null 和 undefined', () => {
      expect(pathNormalizerSerializer.test(null)).toBe(false);
      expect(pathNormalizerSerializer.test(undefined)).toBe(false);
    });

    test('应该处理空对象和空数组', () => {
      expect(pathNormalizerSerializer.test({})).toBe(false);
      expect(pathNormalizerSerializer.test([])).toBe(false);
    });

    test('应该处理包含特殊字符的路径', () => {
      const specialPaths = [
        'C:\\Users\\用户名\\项目\\文件.txt',
        '/home/user with spaces/project',
        'D:\\Projects\\app-with-dashes\\src'
      ];
      
      specialPaths.forEach(path => {
        expect(pathNormalizerSerializer.test(path)).toBe(true);
        const result = pathNormalizerSerializer.serialize(path);
        expect(typeof result).toBe('string');
      });
    });

    test('应该处理深层嵌套结构', () => {
      const deepNested = {
        level1: {
          level2: {
            level3: {
              paths: [
                originalCwd + '/deep/nested/file.js',
                'C:\\Deep\\Nested\\File.exe'
              ]
            }
          }
        }
      };
      
      expect(pathNormalizerSerializer.test(deepNested)).toBe(true);
      
      const result = JSON.parse(pathNormalizerSerializer.serialize(deepNested));
      expect(result.level1.level2.level3.paths[0]).toBe('<ROOT>/deep/nested/file.js');
      expect(result.level1.level2.level3.paths[1]).toBe('<DRIVE>/Deep/Nested/File.exe');
    });
  });
});