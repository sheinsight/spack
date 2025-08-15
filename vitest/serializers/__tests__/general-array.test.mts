/**
 * 通用数组序列化器单元测试
 */

import { describe, test, expect } from 'vitest';
import { generalArraySerializer } from '../general-array.mts';

describe('generalArraySerializer', () => {
  describe('test() 方法 - 检测逻辑', () => {
    test('应该检测数组类型', () => {
      expect(generalArraySerializer.test([])).toBe(true);
      expect(generalArraySerializer.test(['a', 'b', 'c'])).toBe(true);
      expect(generalArraySerializer.test([1, 2, 3])).toBe(true);
      expect(generalArraySerializer.test([{}, {}, {}])).toBe(true);
    });

    test('应该忽略非数组类型', () => {
      expect(generalArraySerializer.test('string')).toBe(false);
      expect(generalArraySerializer.test(123)).toBe(false);
      expect(generalArraySerializer.test({})).toBe(false);
      expect(generalArraySerializer.test(null)).toBe(false);
      expect(generalArraySerializer.test(undefined)).toBe(false);
      expect(generalArraySerializer.test(true)).toBe(false);
    });
  });

  describe('serialize() 方法 - 序列化逻辑', () => {
    test('应该对字符串数组排序', () => {
      const input = ['zebra', 'apple', 'banana', 'cherry'];
      const result = JSON.parse(generalArraySerializer.serialize(input));
      
      expect(result).toEqual(['apple', 'banana', 'cherry', 'zebra']);
    });

    test('应该对数字数组排序', () => {
      const input = [42, 1, 99, 7, 23];
      const result = JSON.parse(generalArraySerializer.serialize(input));
      
      expect(result).toEqual([1, 7, 23, 42, 99]);
    });

    test('应该对布尔值数组排序', () => {
      const input = [false, true, false, true];
      const result = JSON.parse(generalArraySerializer.serialize(input));
      
      expect(result).toEqual([false, false, true, true]);
    });

    test('应该处理混合类型数组', () => {
      const input = ['string', 42, true, null, undefined, false, 'another'];
      const result = JSON.parse(generalArraySerializer.serialize(input));
      
      // 按类型排序: boolean, number, object (null), string, undefined (变为 null)
      expect(result).toEqual([false, true, 42, null, 'another', 'string', null]);
    });

    test('应该对对象数组排序', () => {
      const input = [
        { name: 'zebra', id: 3 },
        { name: 'apple', id: 1 },
        { name: 'banana', id: 2 }
      ];
      const result = JSON.parse(generalArraySerializer.serialize(input));
      
      // 对象按 JSON 字符串排序
      expect(result[0].name).toBe('apple');
      expect(result[1].name).toBe('banana');
      expect(result[2].name).toBe('zebra');
    });

    test('应该递归处理嵌套数组', () => {
      const input = [
        ['zebra', 'apple'],
        ['delta', 'alpha', 'beta'],
        ['one']
      ];
      const result = JSON.parse(generalArraySerializer.serialize(input));
      
      // 外层数组排序，内层数组也排序
      expect(result).toEqual([
        ['alpha', 'beta', 'delta'],
        ['apple', 'zebra'],
        ['one']
      ]);
    });

    test('应该递归处理包含对象的数组', () => {
      const input = [
        {
          name: 'project-b',
          files: ['z.js', 'a.js', 'b.js']
        },
        {
          name: 'project-a',
          files: ['y.js', 'x.js']
        }
      ];
      const result = JSON.parse(generalArraySerializer.serialize(input));
      
      // 对象数组按 JSON 排序，内部数组也排序
      expect(result[0].name).toBe('project-a');
      expect(result[0].files).toEqual(['x.js', 'y.js']);
      expect(result[1].name).toBe('project-b');
      expect(result[1].files).toEqual(['a.js', 'b.js', 'z.js']);
    });

    test('应该处理空数组', () => {
      const result = JSON.parse(generalArraySerializer.serialize([]));
      expect(result).toEqual([]);
    });

    test('应该处理包含重复元素的数组', () => {
      const input = ['banana', 'apple', 'banana', 'apple', 'cherry'];
      const result = JSON.parse(generalArraySerializer.serialize(input));
      
      expect(result).toEqual(['apple', 'apple', 'banana', 'banana', 'cherry']);
    });
  });

  describe('复杂场景', () => {
    test('应该处理深层嵌套的混合数据结构', () => {
      const input = [
        {
          type: 'config',
          items: [
            {
              name: 'zebra-config',
              values: [3, 1, 4]
            },
            {
              name: 'alpha-config',
              values: ['z', 'a', 'b']
            }
          ]
        },
        {
          type: 'data',
          items: [
            {
              name: 'beta-data',
              values: [true, false]
            }
          ]
        }
      ];
      
      const result = JSON.parse(generalArraySerializer.serialize(input));
      
      // 顶层按 JSON 排序，type: 'config' 和 'data'
      expect(result[0].type).toBe('config');
      expect(result[1].type).toBe('data');
      
      // config.items 按名称排序
      expect(result[0].items[0].name).toBe('alpha-config');
      expect(result[0].items[1].name).toBe('zebra-config');
      
      // 各自的 values 也排序
      expect(result[0].items[0].values).toEqual(['a', 'b', 'z']);
      expect(result[0].items[1].values).toEqual([1, 3, 4]);
      expect(result[1].items[0].values).toEqual([false, true]);
    });

    test('应该处理包含特殊值的数组', () => {
      const input = [
        'normal string',
        '',
        'string with spaces',
        'string-with-dashes',
        'string_with_underscores',
        '123numeric',
        'SpecialChars!@#'
      ];
      
      const result = JSON.parse(generalArraySerializer.serialize(input));
      
      // 字符串按字典序排序
      expect(result[0]).toBe('');
      expect(result[1]).toBe('123numeric');
      expect(result[2]).toBe('normal string');
      expect(result[3]).toBe('SpecialChars!@#');
    });

    test('应该处理包含中文和特殊字符的数组', () => {
      const input = [
        '中文',
        'English',
        '日本語',
        '한국어',
        'Français',
        'Español'
      ];
      
      const result = JSON.parse(generalArraySerializer.serialize(input));
      
      // 应该按 Unicode 排序
      expect(result).toHaveLength(6);
      expect(result.includes('中文')).toBe(true);
      expect(result.includes('English')).toBe(true);
    });

    test('应该处理数值的边缘情况', () => {
      const input = [
        Infinity,
        -Infinity,
        NaN,
        0,
        -0,
        Number.MAX_VALUE,
        Number.MIN_VALUE
      ];
      
      const result = JSON.parse(generalArraySerializer.serialize(input));
      
      // 数字应该正确排序，但特殊值在 JSON.stringify 中变为 null
      expect(result).toHaveLength(7);
      expect(result.includes(null)).toBe(true); // -Infinity, Infinity, NaN 都变成 null
    });
  });

  describe('边缘情况', () => {
    test('应该处理稀疏数组', () => {
      const input = [, 'second', , 'fourth'];  // 稀疏数组
      const result = JSON.parse(generalArraySerializer.serialize(input));
      
      // undefined 元素在 JSON.stringify 中变为 null
      expect(result.includes('second')).toBe(true);
      expect(result.includes('fourth')).toBe(true);
      expect(result.includes(null)).toBe(true); // undefined 变成 null
    });

    test('应该处理类数组对象（不应该匹配）', () => {
      const arrayLike = {
        0: 'first',
        1: 'second',
        length: 2
      };
      
      // 类数组对象不应该被处理
      expect(generalArraySerializer.test(arrayLike)).toBe(false);
    });

    test('应该处理函数数组', () => {
      const func1 = function a() {};
      const func2 = function b() {};
      const func3 = () => {};
      
      const input = [func2, func1, func3];
      const result = JSON.parse(generalArraySerializer.serialize(input));
      
      // 函数会被转换为字符串进行排序
      expect(result).toHaveLength(3);
    });

    test('应该处理 Symbol 类型（会被过滤）', () => {
      const sym1 = Symbol('first');
      const sym2 = Symbol('second');
      
      const input = ['string', sym1, 'another', sym2];
      
      // Symbol 在 JSON.stringify 中会被忽略，但数组长度保持
      const result = JSON.parse(generalArraySerializer.serialize(input));
      
      expect(result).toEqual(['another', 'string', null, null]); // Symbol 变成 null
    });

    test('应该保持排序的稳定性', () => {
      const input = [
        { id: 1, name: 'same' },
        { id: 2, name: 'same' },
        { id: 3, name: 'same' }
      ];
      
      const result = JSON.parse(generalArraySerializer.serialize(input));
      
      // 相同内容的对象应该按其原始 JSON 字符串排序
      expect(result).toHaveLength(3);
      expect(result[0].id).toBe(1);
      expect(result[1].id).toBe(2);
      expect(result[2].id).toBe(3);
    });
  });
});