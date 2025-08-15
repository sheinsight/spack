/**
 * 统一的快照序列化器管理
 * 按优先级组合各种专用序列化器
 */

import type { SnapshotSerializer } from 'vitest';
import { pathNormalizerSerializer } from './path-normalizer.mts';
import { duplicateDependencySerializer } from './duplicate-dependency.mts';
import { errorsArraySerializer } from './errors-array.mts';
import { generalArraySerializer } from './general-array.mts';

/**
 * 序列化器优先级列表
 * 越前面的优先级越高
 */
const serializers: SnapshotSerializer[] = [
  pathNormalizerSerializer,  // 路径标准化应该最优先
  duplicateDependencySerializer,
  errorsArraySerializer,
  generalArraySerializer,
];

/**
 * 统一快照序列化器
 * 按优先级匹配并应用相应的序列化逻辑
 */
const unifiedSerializer: SnapshotSerializer = {
  test: (val: unknown): boolean => {
    // 检查是否有任何序列化器可以处理这个值
    return serializers.some((serializer) => serializer.test(val));
  },

  serialize: (val: any) => {
    // 按优先级应用序列化逻辑
    for (const serializer of serializers) {
      if (serializer.test(val)) {
        return (serializer as SnapshotSerializer).serialize(val);
      }
    }

    // 默认情况，不应该到达这里
    return JSON.stringify(val, null, 2);
  },
};

export default unifiedSerializer;
