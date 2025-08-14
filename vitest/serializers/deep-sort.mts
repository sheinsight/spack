/**
 * 深度排序工具函数
 * 递归处理嵌套数组和对象的排序
 */

/**
 * 深度排序任意数据结构
 * @param obj 需要排序的对象或数组
 * @returns 排序后的数据结构
 */
export function deepSort(obj: any): any {
  if (Array.isArray(obj)) {
    // 先递归排序数组内的每个元素
    const sortedItems = obj.map(item => deepSort(item));
    
    // 然后对数组本身排序
    return sortedItems.sort((a: any, b: any) => {
      if (typeof a === 'string' && typeof b === 'string') {
        return a.localeCompare(b);
      }
      if (typeof a === 'number' && typeof b === 'number') {
        return a - b;
      }
      if (typeof a === 'object' && typeof b === 'object' && a !== null && b !== null) {
        // 对象按 JSON 字符串排序
        return JSON.stringify(a).localeCompare(JSON.stringify(b));
      }
      // 混合类型按类型名称排序，然后按字符串表示排序
      const typeA = typeof a;
      const typeB = typeof b;
      if (typeA !== typeB) {
        return typeA.localeCompare(typeB);
      }
      return String(a).localeCompare(String(b));
    });
  } else if (typeof obj === 'object' && obj !== null) {
    // 对象的每个属性值都递归处理
    const result: any = {};
    for (const [key, value] of Object.entries(obj)) {
      result[key] = deepSort(value);
    }
    return result;
  } else {
    // 基本类型直接返回
    return obj;
  }
}