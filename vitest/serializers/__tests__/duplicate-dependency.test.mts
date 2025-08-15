/**
 * 重复依赖检测序列化器单元测试
 */

import { describe, test, expect } from 'vitest';
import { duplicateDependencySerializer } from '../duplicate-dependency.mts';

describe('duplicateDependencySerializer', () => {
  describe('test() 方法 - 检测逻辑', () => {
    test('应该检测包含 groups 数组的对象', () => {
      const validData = {
        groups: [
          { name: 'react', versions: ['16.0.0', '17.0.0'] }
        ]
      };
      
      expect(duplicateDependencySerializer.test(validData)).toBe(true);
    });

    test('应该忽略不包含 groups 属性的对象', () => {
      const invalidData = {
        dependencies: ['react', 'lodash'],
        count: 5
      };
      
      expect(duplicateDependencySerializer.test(invalidData)).toBe(false);
    });

    test('应该忽略 groups 不是数组的对象', () => {
      const invalidData = {
        groups: 'not-an-array'
      };
      
      expect(duplicateDependencySerializer.test(invalidData)).toBe(false);
    });

    test('应该处理空的 groups 数组', () => {
      const validData = {
        groups: []
      };
      
      expect(duplicateDependencySerializer.test(validData)).toBe(true);
    });

    test('应该忽略非对象类型', () => {
      expect(duplicateDependencySerializer.test('string')).toBe(false);
      expect(duplicateDependencySerializer.test(123)).toBe(false);
      expect(duplicateDependencySerializer.test(null)).toBe(false);
      expect(duplicateDependencySerializer.test(undefined)).toBe(false);
      expect(duplicateDependencySerializer.test([])).toBe(false);
    });
  });

  describe('serialize() 方法 - 序列化逻辑', () => {
    test('应该对简单的 groups 数据进行深度排序', () => {
      const input = {
        groups: [
          {
            name: 'lodash',
            versions: ['4.17.20', '4.17.19']
          },
          {
            name: 'react',
            versions: ['17.0.0', '16.0.0']
          }
        ]
      };
      
      const result = JSON.parse(duplicateDependencySerializer.serialize(input));
      
      // groups 数组应该按对象的 JSON 字符串排序
      expect(result.groups[0].name).toBe('lodash');
      expect(result.groups[1].name).toBe('react');
      
      // 每个 group 内的 versions 数组也应该排序
      expect(result.groups[0].versions).toEqual(['4.17.19', '4.17.20']);
      expect(result.groups[1].versions).toEqual(['16.0.0', '17.0.0']);
    });

    test('应该处理复杂的嵌套结构', () => {
      const input = {
        groups: [
          {
            name: 'react',
            versions: ['17.0.0', '16.0.0'],
            locations: [
              {
                path: '/node_modules/react',
                dependencies: ['object-assign', 'prop-types']
              },
              {
                path: '/node_modules/app/node_modules/react',
                dependencies: ['scheduler', 'object-assign']
              }
            ]
          }
        ],
        metadata: {
          totalGroups: 1,
          scanPaths: ['/project/node_modules', '/project/src']
        }
      };
      
      const result = JSON.parse(duplicateDependencySerializer.serialize(input));
      
      // versions 应该排序
      expect(result.groups[0].versions).toEqual(['16.0.0', '17.0.0']);
      
      // locations 数组应该排序，并且内部的 dependencies 也应该排序
      const locations = result.groups[0].locations;
      expect(locations).toHaveLength(2);
      locations.forEach((location: any) => {
        expect(location.dependencies).toEqual([...location.dependencies].sort());
      });
      
      // metadata.scanPaths 应该排序
      expect(result.metadata.scanPaths).toEqual(['/project/node_modules', '/project/src']);
    });

    test('应该处理包含不同数据类型的混合数据', () => {
      const input = {
        groups: [
          {
            name: 'package-b',
            versions: ['2.0.0', '1.0.0'],
            priority: 2,
            isDevDependency: false
          },
          {
            name: 'package-a',
            versions: ['1.5.0'],
            priority: 1,
            isDevDependency: true
          }
        ],
        stats: {
          duplicates: 2,
          affected: ['app', 'utils', 'components']
        }
      };
      
      const result = JSON.parse(duplicateDependencySerializer.serialize(input));
      
      // groups 按 JSON 字符串排序
      expect(result.groups[0].name).toBe('package-a'); // 'package-a' 在 JSON 字符串中排在前面
      expect(result.groups[1].name).toBe('package-b');
      
      // versions 排序
      expect(result.groups[1].versions).toEqual(['1.0.0', '2.0.0']);
      
      // stats.affected 数组排序
      expect(result.stats.affected).toEqual(['app', 'components', 'utils']);
    });

    test('应该创建深拷贝，不修改原始数据', () => {
      const originalInput = {
        groups: [
          {
            name: 'test',
            versions: ['3.0.0', '1.0.0', '2.0.0']
          }
        ]
      };
      
      // 保存原始数据的快照
      const originalVersions = [...originalInput.groups[0].versions];
      
      duplicateDependencySerializer.serialize(originalInput);
      
      // 原始数据不应该被修改
      expect(originalInput.groups[0].versions).toEqual(originalVersions);
    });

    test('应该处理空的 groups 数组', () => {
      const input = {
        groups: [],
        summary: 'no duplicates found'
      };
      
      const result = JSON.parse(duplicateDependencySerializer.serialize(input));
      
      expect(result.groups).toEqual([]);
      expect(result.summary).toBe('no duplicates found');
    });
  });

  describe('边缘情况', () => {
    test('应该处理包含 null/undefined 值的数据', () => {
      const input = {
        groups: [
          {
            name: 'package',
            versions: ['1.0.0', null, '2.0.0', undefined],
            metadata: null
          }
        ]
      };
      
      const result = JSON.parse(duplicateDependencySerializer.serialize(input));
      
      // null 和 undefined 在数组中保留为 null
      expect(result.groups[0].versions).toEqual([null, null, '1.0.0', '2.0.0']);
      expect(result.groups[0].metadata).toBe(null);
    });

    test('应该处理包含特殊字符的包名', () => {
      const input = {
        groups: [
          {
            name: '@org/package-name',
            versions: ['1.0.0']
          },
          {
            name: 'regular-package',
            versions: ['2.0.0']
          }
        ]
      };
      
      const result = JSON.parse(duplicateDependencySerializer.serialize(input));
      
      // 特殊字符应该被正确处理
      expect(result.groups).toHaveLength(2);
      expect(result.groups.some((g: any) => g.name === '@org/package-name')).toBe(true);
    });

    test('应该处理深层嵌套的数组结构', () => {
      const input = {
        groups: [
          {
            name: 'complex-package',
            conflicts: [
              {
                type: 'version',
                details: [
                  ['file3.js', 'file1.js', 'file2.js'],
                  ['moduleC', 'moduleA', 'moduleB']
                ]
              }
            ]
          }
        ]
      };
      
      const result = JSON.parse(duplicateDependencySerializer.serialize(input));
      
      // 深层嵌套的数组应该都被排序
      const details = result.groups[0].conflicts[0].details;
      expect(details[0]).toEqual(['file1.js', 'file2.js', 'file3.js']);
      expect(details[1]).toEqual(['moduleA', 'moduleB', 'moduleC']);
    });
  });
});