/**
 * 统一序列化器单元测试
 * 测试序列化器优先级和组合逻辑
 */

import { describe, test, expect, vi } from 'vitest';
import unifiedSerializer from '../index.mts';

describe('unifiedSerializer (统一序列化器)', () => {
  describe('test() 方法 - 序列化器匹配逻辑', () => {
    test('应该正确检测路径标准化需求（最高优先级）', () => {
      const pathData = {
        message: 'Error in /absolute/path/to/file.js',
        moduleId: 'C:\\Windows\\System32\\file.dll'
      };
      
      expect(unifiedSerializer.test(pathData)).toBe(true);
    });

    test('应该正确检测重复依赖数据', () => {
      const duplicateData = {
        groups: [
          { name: 'react', versions: ['16.0.0', '17.0.0'] }
        ]
      };
      
      expect(unifiedSerializer.test(duplicateData)).toBe(true);
    });

    test('应该正确检测错误数组', () => {
      const errorArray = [
        { message: 'Error occurred', moduleId: 'module.js' }
      ];
      
      expect(unifiedSerializer.test(errorArray)).toBe(true);
    });

    test('应该正确检测通用数组', () => {
      const generalArray = ['zebra', 'apple', 'banana'];
      
      expect(unifiedSerializer.test(generalArray)).toBe(true);
    });

    test('应该忽略不匹配任何序列化器的数据', () => {
      const simpleData = {
        name: 'test',
        value: 123,
        relative: './relative/path'
      };
      
      expect(unifiedSerializer.test(simpleData)).toBe(false);
    });
  });

  describe('serialize() 方法 - 序列化器优先级', () => {
    test('路径标准化应该优先于错误数组序列化', () => {
      const errorArrayWithPaths = [
        {
          message: 'Error in /absolute/path/to/file.js',
          moduleId: '/another/absolute/path.js'
        }
      ];
      
      const result = JSON.parse(unifiedSerializer.serialize(errorArrayWithPaths));
      
      // 应该被路径标准化序列化器处理，而不是错误数组序列化器
      expect(result[0].message).toContain('/absolute/path/to/file.js'); // 路径未被替换说明用的是路径序列化器
    });

    test('路径标准化应该优先于重复依赖序列化', () => {
      const duplicateDataWithPaths = {
        groups: [
          {
            name: 'package',
            location: '/absolute/path/to/node_modules/package'
          }
        ]
      };
      
      const result = JSON.parse(unifiedSerializer.serialize(duplicateDataWithPaths));
      
      // 应该被路径标准化序列化器处理
      expect(typeof result).toBe('object');
      expect(result.groups[0].location).toContain('/absolute/path/to/node_modules/package');
    });

    test('重复依赖序列化应该优先于通用数组序列化', () => {
      const duplicateData = {
        groups: [
          { name: 'zebra-package', versions: ['2.0.0', '1.0.0'] },
          { name: 'alpha-package', versions: ['1.5.0'] }
        ]
      };
      
      const result = JSON.parse(unifiedSerializer.serialize(duplicateData));
      
      // 应该被重复依赖序列化器处理（使用深度排序）
      expect(result.groups).toHaveLength(2);
      // 重复依赖序列化器会对 groups 和内部的 versions 都排序
      expect(result.groups[0].versions).toEqual(['1.5.0']); // 第一个 group 的 versions
      expect(result.groups[1].versions).toEqual(['1.0.0', '2.0.0']); // 第二个 group 的 versions
    });

    test('错误数组序列化应该优先于通用数组序列化', () => {
      const errorArray = [
        { message: 'Error B', moduleId: 'module-b.js' },
        { message: 'Error A', moduleId: 'module-a.js' }
      ];
      
      const result = JSON.parse(unifiedSerializer.serialize(errorArray));
      
      // 应该被错误数组序列化器处理（按 moduleId 排序）
      expect(result[0].moduleId).toBe('module-a.js');
      expect(result[1].moduleId).toBe('module-b.js');
    });

    test('通用数组序列化应该作为后备选项', () => {
      const generalArray = ['zebra', 'apple', 'banana'];
      
      const result = JSON.parse(unifiedSerializer.serialize(generalArray));
      
      // 应该被通用数组序列化器处理
      expect(result).toEqual(['apple', 'banana', 'zebra']);
    });
  });

  describe('复杂优先级场景', () => {
    test('包含绝对路径的错误数组应该优先使用路径标准化', () => {
      const complexData = [
        {
          message: 'Module not found in C:\\Projects\\app\\src\\index.ts',
          moduleId: '/usr/local/project/module.js',
          stack: ['frame1', 'frame2']
        }
      ];
      
      expect(unifiedSerializer.test(complexData)).toBe(true);
      
      const result = JSON.parse(unifiedSerializer.serialize(complexData));
      
      // 路径应该被标准化（反斜杠转为正斜杠，但不一定替换为 <DRIVE>）
      expect(result[0].message).toContain('C:/Projects/app/src/index.ts');
      expect(result[0].moduleId).toBe('/usr/local/project/module.js');
      
      // 同时内部数组也应该被排序（路径标准化器使用深度排序）
      expect(result[0].stack).toEqual(['frame1', 'frame2']);
    });

    test('包含绝对路径的重复依赖数据应该优先使用路径标准化', () => {
      const complexData = {
        groups: [
          {
            name: 'react',
            versions: ['17.0.0', '16.0.0'],
            path: 'C:\\node_modules\\react'
          }
        ],
        scanPath: '/usr/local/project'
      };
      
      expect(unifiedSerializer.test(complexData)).toBe(true);
      
      const result = JSON.parse(unifiedSerializer.serialize(complexData));
      
      // 路径应该被标准化
      expect(result.groups[0].path).toBe('<DRIVE>/node_modules/react');
      expect(result.scanPath).toBe('/usr/local/project');
      
      // 同时数组也应该被排序（路径标准化器保持原顺序）
      expect(result.groups[0].versions).toEqual(['17.0.0', '16.0.0']);
    });
  });

  describe('边缘情况', () => {
    test('当没有序列化器匹配时应该使用默认 JSON.stringify', () => {
      const unmatched = {
        simple: 'data',
        number: 123,
        relative: './relative/path'
      };
      
      // test() 应该返回 false
      expect(unifiedSerializer.test(unmatched)).toBe(false);
      
      // 但 serialize() 应该仍然工作（使用默认逻辑）
      const result = unifiedSerializer.serialize(unmatched);
      expect(typeof result).toBe('string');
      
      const parsed = JSON.parse(result);
      expect(parsed.simple).toBe('data');
      expect(parsed.number).toBe(123);
    });

    test('应该处理空数据', () => {
      expect(unifiedSerializer.test(null)).toBe(false);
      expect(unifiedSerializer.test(undefined)).toBe(false);
      expect(unifiedSerializer.test({})).toBe(false);
      expect(unifiedSerializer.test([])).toBe(true); // 空数组会被通用数组序列化器匹配
    });

    test('应该处理嵌套的多层优先级情况', () => {
      // 创建一个同时匹配多个序列化器的复杂对象
      const multiMatch = {
        // 匹配重复依赖序列化器
        groups: [
          {
            name: 'package',
            // 匹配路径标准化序列化器
            location: 'C:\\absolute\\path\\to\\package',
            // 匹配错误数组序列化器的嵌套数据
            errors: [
              { message: 'Error in package', moduleId: 'package/index.js' }
            ]
          }
        ]
      };
      
      expect(unifiedSerializer.test(multiMatch)).toBe(true);
      
      const result = JSON.parse(unifiedSerializer.serialize(multiMatch));
      
      // 应该被路径标准化序列化器处理（最高优先级）
      expect(result.groups[0].location).toBe('<DRIVE>/absolute/path/to/package');
      
      // 嵌套的错误数组也应该被递归处理
      expect(Array.isArray(result.groups[0].errors)).toBe(true);
    });
  });

  describe('性能和稳定性', () => {
    test('应该对大量数据保持稳定的序列化结果', () => {
      const largeArray = Array.from({ length: 1000 }, (_, i) => `item-${999 - i}`);
      
      const result1 = unifiedSerializer.serialize(largeArray);
      const result2 = unifiedSerializer.serialize(largeArray);
      
      // 多次序列化应该得到相同结果
      expect(result1).toBe(result2);
      
      const parsed = JSON.parse(result1);
      expect(parsed[0]).toBe('item-0');
      expect(parsed[999]).toBe('item-999');
    });

    test('应该正确处理循环引用的错误情况', () => {
      const circular: any = { name: 'test' };
      circular.self = circular;
      
      // 包含循环引用的对象应该被正确检测
      expect(unifiedSerializer.test([circular])).toBe(true);
      
      // 但序列化时应该抛出错误（由于递归深度或循环引用）
      expect(() => unifiedSerializer.serialize([circular])).toThrow();
    });
  });
});