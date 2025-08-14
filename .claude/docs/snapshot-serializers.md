# 快照序列化器架构

项目使用模块化的快照序列化器系统，基于单一职责原则设计。

## 目录结构

```
vitest/
├── setup.mts                    # Vitest 测试设置文件
└── serializers/
    ├── index.mts                # 统一入口和优先级管理
    ├── deep-sort.mts            # 深度排序工具函数
    ├── path-normalizer.mts      # 路径标准化序列化器
    ├── duplicate-dependency.mts # 重复依赖检测序列化器
    ├── errors-array.mts         # 错误数组序列化器
    └── general-array.mts        # 通用数组序列化器
```

## 设计原则

### 单一职责原则
每个序列化器文件只负责一种特定类型的数据序列化：

- **`deep-sort.mts`**: 纯工具函数，专门处理深度递归排序
- **`path-normalizer.mts`**: 专门处理文件路径标准化
- **`duplicate-dependency.mts`**: 专门处理重复依赖检测结果
- **`errors-array.mts`**: 专门处理错误对象数组
- **`general-array.mts`**: 处理通用数组类型

### 优先级机制
序列化器按优先级顺序在 `index.mts` 中组合：

1. `duplicateDependencySerializer` - 最高优先级（特定业务逻辑）
2. `errorsArraySerializer` - 高优先级（特定错误格式）  
3. `pathNormalizerSerializer` - 中优先级（路径处理）
4. `generalArraySerializer` - 最低优先级（通用数组）

### 配置集成
- vitest.config.mts 直接引用 `./vitest/serializers/index.mts` 作为快照序列化器
- setup.mts 位于 `./vitest/setup.mts`，用于测试环境设置
- `deepSort` 工具函数处理深度递归排序，确保嵌套数组/对象的一致性

## 如何添加新序列化器

1. 在 `vitest/serializers/` 目录下创建新文件：
   ```typescript
   import type { SnapshotSerializer } from 'vitest';
   
   export const myCustomSerializer: SnapshotSerializer = {
     test: (val: unknown): val is MyType => {
       // 检测逻辑
     },
     serialize: (val: MyType) => {
       // 序列化逻辑
       return JSON.stringify(val, null, 2);
     }
   };
   ```

2. 在 `index.mts` 中导入并添加到优先级列表：
   ```typescript
   import { myCustomSerializer } from './my-custom.mts';
   
   const serializers = [
     myCustomSerializer,  // 根据需要调整优先级位置
     duplicateDependencySerializer,
     // ... 其他序列化器
   ];
   ```

## 优势

- **可维护性**: 每个文件职责清晰，易于理解和修改
- **可测试性**: 可以单独测试每个序列化器
- **可扩展性**: 添加新序列化器无需修改现有代码
- **复用性**: `deepSort` 工具函数可被多个序列化器复用
- **清晰的依赖关系**: 通过 import/export 清楚看到模块间依赖