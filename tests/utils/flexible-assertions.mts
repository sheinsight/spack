import { expect } from 'vitest';

// 灵活的重复依赖检测断言
export function expectDuplicateDependencies(result: any, expectedDuplicates: { name: string; versions: string[] }[]) {
  expect(result).toHaveProperty('groups');
  expect(Array.isArray(result.groups)).toBe(true);
  expect(result.groups).toHaveLength(expectedDuplicates.length);
  
  for (const expected of expectedDuplicates) {
    const group = result.groups.find((g: any) => g.name === expected.name);
    expect(group).toBeDefined();
    expect(group.libs).toHaveLength(expected.versions.length);
    
    // 检查版本存在，但不关心顺序
    const actualVersions = group.libs.map((lib: any) => lib.version).sort();
    const expectedVersionsSorted = [...expected.versions].sort();
    expect(actualVersions).toEqual(expectedVersionsSorted);
  }
}

// 灵活的错误断言
export function expectErrorsContain(errors: any[], expectedPatterns: string[]) {
  expect(Array.isArray(errors)).toBe(true);
  
  for (const pattern of expectedPatterns) {
    const hasMatchingError = errors.some(error => 
      error.message && error.message.includes(pattern)
    );
    expect(hasMatchingError).toBe(true);
  }
}

// 期望特定类型的错误
export function expectErrorType(errors: any[], errorType: string, count?: number) {
  expect(Array.isArray(errors)).toBe(true);
  
  const matchingErrors = errors.filter(error => error.code === errorType);
  if (count !== undefined) {
    expect(matchingErrors).toHaveLength(count);
  } else {
    expect(matchingErrors.length).toBeGreaterThan(0);
  }
  
  return matchingErrors;
}