// 统一的快照序列化器管理
// 处理路径标准化、输出排序等问题

// 1. 路径标准化序列化器 - 将绝对路径替换为 <ROOT>
const pathNormalizerSerializer = {
  test: (val: unknown): val is string => 
    typeof val === 'string' && val.includes(process.cwd()),
  serialize: (val: string) => {
    return `"${val.replaceAll(process.cwd(), '<ROOT>')}"`;
  },
};

// 2. 重复依赖检测结果序列化器 - 按文件路径排序
const duplicateDependencySerializer = {
  test: (val: unknown): val is object => {
    return typeof val === 'object' && 
           val !== null && 
           'groups' in val && 
           Array.isArray((val as any).groups);
  },
  serialize: (val: any) => {
    // 深拷贝并排序
    const sorted = JSON.parse(JSON.stringify(val));
    
    if (sorted.groups && Array.isArray(sorted.groups)) {
      sorted.groups.forEach((group: any) => {
        if (group.libs && Array.isArray(group.libs)) {
          group.libs.sort((a: any, b: any) => 
            (a.file || '').localeCompare(b.file || '')
          );
        }
      });
      sorted.groups.sort((a: any, b: any) => 
        (a.name || '').localeCompare(b.name || '')
      );
    }
    
    return JSON.stringify(sorted, null, 2);
  }
};

// 3. 错误数组序列化器 - 按 moduleId/message 排序
const errorsArraySerializer = {
  test: (val: unknown): val is Array<any> => {
    return Array.isArray(val) && 
           val.some(item => typeof item === 'object' && 
                           item !== null && 
                           ('moduleId' in item || 'message' in item));
  },
  serialize: (val: any[]) => {
    const sorted = [...val].sort((a: any, b: any) => {
      const keyA = a.moduleId || a.message || '';
      const keyB = b.moduleId || b.message || '';
      return keyA.localeCompare(keyB);
    });
    
    return JSON.stringify(sorted, null, 2);
  }
};

// 统一序列化器 - 按优先级处理不同类型的数据
export default {
  test: (val: unknown): boolean => {
    // 按优先级检查各种类型
    return duplicateDependencySerializer.test(val) ||
           errorsArraySerializer.test(val) ||
           pathNormalizerSerializer.test(val);
  },
  serialize: (val: any) => {
    // 按优先级应用序列化逻辑
    if (duplicateDependencySerializer.test(val)) {
      return duplicateDependencySerializer.serialize(val);
    }
    if (errorsArraySerializer.test(val)) {
      return errorsArraySerializer.serialize(val);
    }
    if (pathNormalizerSerializer.test(val)) {
      return pathNormalizerSerializer.serialize(val);
    }
    
    // 默认情况，不应该到达这里
    return JSON.stringify(val, null, 2);
  }
};