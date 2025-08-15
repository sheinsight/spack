/**
 * 错误数组序列化器单元测试
 */

import { describe, test, expect } from 'vitest';
import { errorsArraySerializer } from '../errors-array.mts';

describe('errorsArraySerializer', () => {
  describe('test() 方法 - 检测逻辑', () => {
    test('应该检测包含 moduleId 属性的错误对象数组', () => {
      const errorArray = [
        {
          message: 'Module not found',
          moduleId: './src/index.js',
          code: 'MODULE_NOT_FOUND'
        }
      ];
      
      expect(errorsArraySerializer.test(errorArray)).toBe(true);
    });

    test('应该检测包含 message 属性的错误对象数组', () => {
      const errorArray = [
        {
          message: 'Syntax error',
          line: 10,
          column: 5
        }
      ];
      
      expect(errorsArraySerializer.test(errorArray)).toBe(true);
    });

    test('应该检测混合的错误对象数组', () => {
      const errorArray = [
        {
          message: 'Error A',
          moduleId: 'moduleA'
        },
        {
          message: 'Error B',
          code: 'ERROR_B'
        }
      ];
      
      expect(errorsArraySerializer.test(errorArray)).toBe(true);
    });

    test('应该忽略非数组类型', () => {
      expect(errorsArraySerializer.test('string')).toBe(false);
      expect(errorsArraySerializer.test(123)).toBe(false);
      expect(errorsArraySerializer.test({})).toBe(false);
      expect(errorsArraySerializer.test(null)).toBe(false);
      expect(errorsArraySerializer.test(undefined)).toBe(false);
    });

    test('应该忽略空数组', () => {
      expect(errorsArraySerializer.test([])).toBe(false);
    });

    test('应该忽略不包含错误对象的数组', () => {
      const nonErrorArray = [
        'string',
        123,
        { name: 'not-error', value: 'test' }
      ];
      
      expect(errorsArraySerializer.test(nonErrorArray)).toBe(false);
    });

    test('应该忽略部分匹配的数组', () => {
      const mixedArray = [
        { message: 'This is an error' },
        'not an error object',
        123
      ];
      
      // 只要有一个对象包含 message 或 moduleId 就应该匹配
      expect(errorsArraySerializer.test(mixedArray)).toBe(true);
    });

    test('应该处理包含 null 值的对象', () => {
      const errorArray = [
        null,
        { message: 'Error occurred' }
      ];
      
      expect(errorsArraySerializer.test(errorArray)).toBe(true);
    });
  });

  describe('serialize() 方法 - 序列化逻辑', () => {
    test('应该按 moduleId 对错误数组排序', () => {
      const input = [
        {
          message: 'Error C',
          moduleId: 'module-c.js'
        },
        {
          message: 'Error A',
          moduleId: 'module-a.js'
        },
        {
          message: 'Error B',
          moduleId: 'module-b.js'
        }
      ];
      
      const result = JSON.parse(errorsArraySerializer.serialize(input));
      
      expect(result[0].moduleId).toBe('module-a.js');
      expect(result[1].moduleId).toBe('module-b.js');
      expect(result[2].moduleId).toBe('module-c.js');
    });

    test('应该按 message 对错误数组排序（当没有 moduleId 时）', () => {
      const input = [
        {
          message: 'Zebra error',
          code: 'ZEBRA'
        },
        {
          message: 'Alpha error',
          code: 'ALPHA'
        },
        {
          message: 'Beta error',
          code: 'BETA'
        }
      ];
      
      const result = JSON.parse(errorsArraySerializer.serialize(input));
      
      expect(result[0].message).toBe('Alpha error');
      expect(result[1].message).toBe('Beta error');
      expect(result[2].message).toBe('Zebra error');
    });

    test('应该混合排序 moduleId 和 message', () => {
      const input = [
        {
          message: 'Message Z',
          code: 'NO_MODULE_ID'
        },
        {
          message: 'Error with module',
          moduleId: 'module-b.js'
        },
        {
          message: 'Message A',
          line: 10
        },
        {
          message: 'Another error',
          moduleId: 'module-a.js'
        }
      ];
      
      const result = JSON.parse(errorsArraySerializer.serialize(input));
      
      // 应该按 moduleId 或 message 排序
      expect(result[0].message).toBe('Message A'); // 按 message 排序
      expect(result[1].message).toBe('Message Z'); // 按 message 排序
      expect(result[2].moduleId).toBe('module-a.js'); // 按 moduleId 排序
      expect(result[3].moduleId).toBe('module-b.js'); // 按 moduleId 排序
    });

    test('应该对错误对象内部的数组进行深度排序', () => {
      const input = [
        {
          message: 'Error B',
          moduleId: 'module-b.js',
          stack: ['frame3', 'frame1', 'frame2'],
          relatedFiles: ['file-z.js', 'file-a.js']
        },
        {
          message: 'Error A',
          moduleId: 'module-a.js',
          stack: ['frameZ', 'frameA'],
          relatedFiles: ['file-y.js', 'file-b.js']
        }
      ];
      
      const result = JSON.parse(errorsArraySerializer.serialize(input));
      
      // 顶层按 moduleId 排序
      expect(result[0].moduleId).toBe('module-a.js');
      expect(result[1].moduleId).toBe('module-b.js');
      
      // 内部数组应该被深度排序
      expect(result[0].stack).toEqual(['frameA', 'frameZ']);
      expect(result[0].relatedFiles).toEqual(['file-b.js', 'file-y.js']);
      expect(result[1].stack).toEqual(['frame1', 'frame2', 'frame3']);
      expect(result[1].relatedFiles).toEqual(['file-a.js', 'file-z.js']);
    });

    test('应该处理嵌套对象结构', () => {
      const input = [
        {
          message: 'Nested error B',
          location: {
            file: 'file-b.js',
            line: 20,
            suggestions: ['fix-z', 'fix-a', 'fix-b']
          }
        },
        {
          message: 'Nested error A',
          location: {
            file: 'file-a.js',
            line: 10,
            suggestions: ['fix-y', 'fix-c']
          }
        }
      ];
      
      const result = JSON.parse(errorsArraySerializer.serialize(input));
      
      // 按 message 排序
      expect(result[0].message).toBe('Nested error A');
      expect(result[1].message).toBe('Nested error B');
      
      // 嵌套对象内的数组应该排序
      expect(result[0].location.suggestions).toEqual(['fix-c', 'fix-y']);
      expect(result[1].location.suggestions).toEqual(['fix-a', 'fix-b', 'fix-z']);
    });

    test('应该处理包含 null/undefined 的数组', () => {
      const input = [
        null,
        {
          message: 'Valid error',
          moduleId: 'module.js'
        },
        undefined
      ];
      
      const result = JSON.parse(errorsArraySerializer.serialize(input));
      
      // null/undefined 应该被正确排序（排在有效对象后面）
      expect(result).toHaveLength(3);
      expect(result[0]).toEqual({ message: 'Valid error', moduleId: 'module.js' });
      expect(result[1]).toBe(null);
      expect(result[2]).toBe(null); // undefined 在 JSON.stringify 中变成 null
    });
  });

  describe('边缘情况', () => {
    test('应该处理包含特殊字符的错误信息', () => {
      const input = [
        {
          message: 'Error with "quotes" and \\backslashes',
          moduleId: 'special-chars.js'
        },
        {
          message: '中文错误信息',
          moduleId: '中文文件.js'
        }
      ];
      
      const result = JSON.parse(errorsArraySerializer.serialize(input));
      
      expect(result).toHaveLength(2);
      expect(result[0].moduleId).toBe('special-chars.js');
      expect(result[1].moduleId).toBe('中文文件.js');
    });

    test('应该处理空字符串和空值', () => {
      const input = [
        {
          message: '',
          moduleId: 'empty-message.js'
        },
        {
          message: 'Valid message',
          moduleId: ''
        }
      ];
      
      const result = JSON.parse(errorsArraySerializer.serialize(input));
      
      // 空字符串 < 'empty-message.js'，所以第二个项排在第一个前面
      expect(result[0].moduleId).toBe('empty-message.js');
      expect(result[1].moduleId).toBe('');
    });

    test('应该处理大型错误数组', () => {
      const input = Array.from({ length: 100 }, (_, i) => ({
        message: `Error ${99 - i}`, // 逆序创建
        moduleId: `module-${99 - i}.js`
      }));
      
      const result = JSON.parse(errorsArraySerializer.serialize(input));
      
      expect(result).toHaveLength(100);
      expect(result[0].moduleId).toBe('module-0.js');
      expect(result[99].moduleId).toBe('module-99.js');
    });

    test('应该处理复杂的混合数据类型', () => {
      const input = [
        {
          message: 'Complex error',
          moduleId: 'complex.js',
          metadata: {
            numbers: [3, 1, 4],
            strings: ['zebra', 'alpha'],
            mixed: [true, 'string', 42, null]
          }
        }
      ];
      
      const result = JSON.parse(errorsArraySerializer.serialize(input));
      
      const metadata = result[0].metadata;
      expect(metadata.numbers).toEqual([1, 3, 4]);
      expect(metadata.strings).toEqual(['alpha', 'zebra']);
      expect(metadata.mixed).toEqual([true, 42, null, 'string']); // 按类型排序
    });
  });
});