/**
 * 深度排序工具函数单元测试
 */

import { describe, test, expect } from 'vitest';
import { deepSort } from '../deep-sort.mts';

describe('deepSort', () => {
  describe('基本数据类型', () => {
    test('应该直接返回基本类型', () => {
      expect(deepSort('string')).toBe('string');
      expect(deepSort(123)).toBe(123);
      expect(deepSort(true)).toBe(true);
      expect(deepSort(null)).toBe(null);
      expect(deepSort(undefined)).toBe(undefined);
    });
  });

  describe('数组排序', () => {
    test('应该对字符串数组排序', () => {
      const input = ['zebra', 'apple', 'banana'];
      const result = deepSort(input);
      
      expect(result).toEqual(['apple', 'banana', 'zebra']);
    });

    test('应该对数字数组排序', () => {
      const input = [3, 1, 4, 1, 5, 9, 2, 6];
      const result = deepSort(input);
      
      expect(result).toEqual([1, 1, 2, 3, 4, 5, 6, 9]);
    });

    test('应该处理混合类型数组', () => {
      const input = ['string', 123, true, null];
      const result = deepSort(input);
      
      // 按类型排序：boolean, number, object (null), string
      expect(result).toEqual([true, 123, null, 'string']);
    });

    test('应该递归排序嵌套数组', () => {
      const input = [
        ['zebra', 'apple'],
        ['delta', 'charlie', 'bravo'],
        ['one']
      ];
      const result = deepSort(input);
      
      // 数组按 JSON.stringify 结果排序
      expect(result).toEqual([
        ['apple', 'zebra'],  // ["apple","zebra"]
        ['bravo', 'charlie', 'delta'],  // ["bravo","charlie","delta"] 
        ['one']  // ["one"]
      ]);
    });
  });

  describe('对象处理', () => {
    test('应该递归排序对象属性值', () => {
      const input = {
        array: ['zebra', 'apple', 'banana'],
        nested: {
          numbers: [3, 1, 4]
        },
        string: 'unchanged'
      };
      
      const result = deepSort(input);
      
      expect(result).toEqual({
        array: ['apple', 'banana', 'zebra'],
        nested: {
          numbers: [1, 3, 4]
        },
        string: 'unchanged'
      });
    });

    test('应该处理空对象', () => {
      expect(deepSort({})).toEqual({});
    });

    test('应该处理深层嵌套对象', () => {
      const input = {
        level1: {
          level2: {
            level3: {
              items: ['c', 'a', 'b']
            }
          }
        }
      };
      
      const result = deepSort(input);
      
      expect(result.level1.level2.level3.items).toEqual(['a', 'b', 'c']);
    });
  });

  describe('对象数组排序', () => {
    test('应该根据 JSON 字符串对对象数组排序', () => {
      const input = [
        { name: 'zebra', id: 3 },
        { name: 'apple', id: 1 },
        { name: 'banana', id: 2 }
      ];
      
      const result = deepSort(input);
      
      // 对象按 JSON 字符串排序
      expect(result[0].name).toBe('apple');
      expect(result[1].name).toBe('banana');
      expect(result[2].name).toBe('zebra');
    });

    test('应该递归排序对象内的数组', () => {
      const input = [
        {
          name: 'project2',
          files: ['z.js', 'a.js', 'm.js']
        },
        {
          name: 'project1',
          files: ['x.js', 'b.js']
        }
      ];
      
      const result = deepSort(input);
      
      // 每个对象内的 files 数组应该被排序
      result.forEach(project => {
        expect(project.files).toEqual([...project.files].sort());
      });
    });
  });

  describe('复杂场景', () => {
    test('应该处理包含各种数据类型的复杂结构', () => {
      const input = {
        strings: ['zebra', 'apple'],
        numbers: [3, 1, 4],
        objects: [
          { id: 2, name: 'second' },
          { id: 1, name: 'first' }
        ],
        nested: {
          arrays: [
            ['delta', 'alpha'],
            ['beta']
          ]
        },
        mixed: [true, 'string', 42, null]
      };
      
      const result = deepSort(input);
      
      expect(result.strings).toEqual(['apple', 'zebra']);
      expect(result.numbers).toEqual([1, 3, 4]);
      expect(result.objects[0].id).toBe(1); // 按 JSON 排序后，id=1 的在前
      expect(result.nested.arrays[0]).toEqual(['alpha', 'delta']);  // 按 JSON 排序
      expect(result.nested.arrays[1]).toEqual(['beta']);
      expect(result.mixed).toEqual([true, 42, null, 'string']); // 按类型排序
    });

    test('应该处理模拟的测试错误对象', () => {
      const input = [
        {
          message: 'Error B',
          moduleId: 'module2',
          trace: ['frame3', 'frame1', 'frame2']
        },
        {
          message: 'Error A',
          moduleId: 'module1',
          trace: ['frameZ', 'frameA']
        }
      ];
      
      const result = deepSort(input);
      
      // 数组按对象的 JSON 字符串排序
      expect(result[0].message).toBe('Error A');
      expect(result[1].message).toBe('Error B');
      
      // 每个对象内的 trace 数组应该被排序
      expect(result[0].trace).toEqual(['frameA', 'frameZ']);
      expect(result[1].trace).toEqual(['frame1', 'frame2', 'frame3']);
    });

    test('应该处理空数组', () => {
      expect(deepSort([])).toEqual([]);
    });

    test('应该处理包含 undefined 的结构', () => {
      const input = {
        values: [undefined, 'string', undefined, 123]
      };
      
      const result = deepSort(input);
      
      expect(result.values).toEqual([123, 'string', undefined, undefined]);
    });
  });

  describe('边缘情况', () => {
    test('应该处理循环引用 (通过 JSON.stringify 限制)', () => {
      const obj: any = { name: 'test' };
      obj.self = obj;
      
      // deepSort 内部使用 JSON.stringify，会抛出循环引用错误
      expect(() => deepSort([obj, { name: 'other' }])).toThrow();
    });

    test('应该保持排序的稳定性', () => {
      const input = [
        { id: 1, name: 'same' },
        { id: 2, name: 'same' },
        { id: 1, name: 'same' } // 重复对象
      ];
      
      const result = deepSort(input);
      
      // 相同的对象应该按其 JSON 字符串排序
      expect(result).toHaveLength(3);
      expect(result[0].id).toBe(1);
      expect(result[2].id).toBe(2);
    });
  });
});