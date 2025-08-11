// 工具函数：对测试结果进行排序，确保输出一致性
export function sortDuplicateDependencyResult(result: any) {
  if (!result || typeof result !== 'object') {
    return result;
  }

  // 深拷贝避免修改原对象
  const sorted = JSON.parse(JSON.stringify(result));

  if (sorted.groups && Array.isArray(sorted.groups)) {
    sorted.groups.forEach((group: any) => {
      if (group.libs && Array.isArray(group.libs)) {
        // 按文件路径排序，确保顺序一致
        group.libs.sort((a: any, b: any) => {
          return (a.file || '').localeCompare(b.file || '');
        });
      }
    });

    // 对 groups 本身也可以排序
    sorted.groups.sort((a: any, b: any) => {
      return (a.name || '').localeCompare(b.name || '');
    });
  }

  return sorted;
}

// 对错误数组进行排序
export function sortErrorsResult(result: any) {
  if (Array.isArray(result)) {
    return result.sort((a: any, b: any) => {
      // 按 moduleId 或 message 排序
      const keyA = a.moduleId || a.message || '';
      const keyB = b.moduleId || b.message || '';
      return keyA.localeCompare(keyB);
    });
  }
  return result;
}