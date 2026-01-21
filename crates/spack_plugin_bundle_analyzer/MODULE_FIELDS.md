# Module 结构字段扩展分析

本文档分析 Module 数据结构的潜在字段扩展建议。

## 优先级说明

- **高优先级 (High)**: 显著提升分析能力，实现成本合理
- **中优先级 (Medium)**: 有价值但实现成本较高，或使用场景相对受限
- **低优先级 (Low)**: 价值较小或实现成本过高

---

## 当前 Module 结构

```rust
pub struct Module {
  pub id: String,
  pub name: String,
  pub size: u64,
  pub chunks: Vec<String>,
  pub module_kind: ModuleKind,
  pub module_type: ModuleType,
  pub is_node_module: bool,
  pub name_for_condition: String,
  pub concatenated_modules: Option<Vec<ConcatenatedModuleInfo>>,
}
```

---

## 1. dependencies: Vec<String> ⭐ 高优先级

### 实现复杂度

- **复杂度**: 中等
- **实现方式**: 从 rspack 的 `ModuleGraph` 中提取依赖关系
- **代码量**: 约 100-150 行
- **关键 API**: `compilation.module_graph.dependencies(module)`

```rust
// 伪代码
pub fn collect_dependencies(module: &Module, graph: &ModuleGraph) -> Vec<String> {
  graph.dependencies(module.id)
    .map(|dep| dep.module_id.clone())
    .collect()
}
```

### 增加的数据量

- **每个 Module**: 取决于依赖数量
  - 简单模块: 0-5 个依赖，约 20-100 字节
  - 常规模块: 5-20 个依赖，约 100-400 字节
  - 复杂模块: 20-50 个依赖，约 400-1000 字节
- **典型项目** (1000 个模块，平均 10 个依赖): 约 200KB
- **大型项目** (10000 个模块): 约 2MB
- **增长率**: 对总数据量增加约 5-15%

### 性能开销

- **采集阶段**: 低到中等
  - 需要遍历 ModuleGraph
  - 时间复杂度: O(n * m)，n = 模块数，m = 平均依赖数
  - 典型项目: 约 10-50ms
  - 大型项目: 约 100-300ms
- **内存开销**: 中等（需要存储所有依赖关系）
- **传输开销**: 中等（数据量增加 5-15%）

### 可实现功能列表

1. **依赖关系图可视化**:
   - 交互式依赖图
   - 模块间关系探索
   - 依赖路径高亮
2. **关键模块识别**:
   - 找出被依赖最多的模块
   - 计算模块的"中心度"
3. **循环依赖检测**:
   - 自动识别循环引用
   - 展示循环依赖链
4. **影响分析**:
   - 评估修改某模块的影响范围
   - 依赖链深度分析
5. **Tree Shaking 分析**:
   - 识别未使用的依赖
   - 评估 Tree Shaking 效果
6. **重构辅助**:
   - 识别高耦合模块
   - 提供解耦建议
7. **导入路径优化**:
   - 检测不必要的间接依赖
   - 建议更短的导入路径

---

## 2. dependents: Vec<String> ⭐ 高优先级

### 实现复杂度

- **复杂度**: 中等
- **实现方式**: 构建依赖的反向索引
- **代码量**: 约 50-80 行（在收集 dependencies 时同时生成）
- **优化**: 可以与 dependencies 一次性构建，无需二次遍历

```rust
// 伪代码
pub fn build_dependency_graph(modules: &[Module]) -> (DependencyMap, DependentMap) {
  let mut dependencies = HashMap::new();
  let mut dependents = HashMap::new();

  for module in modules {
    for dep in &module.dependencies {
      dependencies.entry(module.id).or_default().push(dep.clone());
      dependents.entry(dep).or_default().push(module.id.clone());
    }
  }

  (dependencies, dependents)
}
```

### 增加的数据量

- **数据量**: 与 dependencies 相同
- **典型项目**: 约 200KB
- **大型项目**: 约 2MB
- **增长率**: 额外增加 5-15%（如果同时添加 dependencies 和 dependents，总增长约 10-30%）

### 性能开销

- **采集阶段**: 几乎为零
  - 如果在构建 dependencies 时同时生成，无额外开销
  - 只是反向索引构建，O(n * m) 复杂度已包含在 dependencies 收集中
- **内存开销**: 与 dependencies 相当
- **传输开销**: 与 dependencies 相同

### 可实现功能列表

1. **影响范围可视化**:
   - 显示修改某模块会影响哪些模块
   - 依赖扇出分析
2. **关键模块识别**:
   - 识别"基础模块"（被很多模块依赖）
   - 模块重要性评分
3. **重构风险评估**:
   - 评估重构某模块的风险级别
   - 计算影响的模块数量
4. **未使用模块检测**:
   - 找出没有 dependents 的模块（除了入口）
   - 识别 Dead Code
5. **模块健康度分析**:
   - 过多 dependents = 可能需要拆分
   - 零 dependents = 可能是死代码
6. **依赖解耦建议**:
   - 识别过度被依赖的模块
   - 提供接口抽象建议
7. **变更影响评估**:
   - CI/CD 中评估变更影响范围
   - 智能测试选择

---

## 3. depth: u32 🔶 中优先级

### 实现复杂度

- **复杂度**: 中等
- **实现方式**: 从入口模块开始进行 BFS 遍历，计算每个模块的最短路径深度
- **代码量**: 约 80-120 行
- **算法**: 广度优先搜索（BFS）

```rust
// 伪代码
pub fn calculate_depths(entry_modules: &[String], graph: &DependencyGraph) -> HashMap<String, u32> {
  let mut depths = HashMap::new();
  let mut queue = VecDeque::new();

  // 初始化入口模块深度为 0
  for entry in entry_modules {
    depths.insert(entry.clone(), 0);
    queue.push_back((entry.clone(), 0));
  }

  // BFS 遍历
  while let Some((module_id, depth)) = queue.pop_front() {
    for dep in graph.dependencies(&module_id) {
      depths.entry(dep.clone()).or_insert_with(|| {
        queue.push_back((dep.clone(), depth + 1));
        depth + 1
      });
    }
  }

  depths
}
```

### 增加的数据量

- **每个 Module**: 4 字节（u32）
- **典型项目** (1000 个模块): 4KB
- **大型项目** (10000 个模块): 40KB
- **增长率**: 对总数据量影响 < 1%

### 性能开销

- **采集阶段**: 低到中等
  - 需要 BFS 遍历依赖图
  - 时间复杂度: O(V + E)，V = 模块数，E = 依赖边数
  - 典型项目: 约 5-20ms
  - 大型项目: 约 50-150ms
- **内存开销**: 低（仅需要队列和深度映射）
- **依赖**: 必须先实现 dependencies 字段

### 可实现功能列表

1. **依赖深度可视化**:
   - 按深度分层展示模块
   - 依赖树层级视图
2. **性能优化建议**:
   - 识别深层依赖（可能影响加载时间）
   - 建议提升常用模块的层级
3. **代码拆分优化**:
   - 深层模块适合异步加载
   - 浅层模块应包含在主 bundle
4. **关键路径分析**:
   - 识别从入口到核心功能的最短路径
   - 优化关键路径上的模块
5. **懒加载候选识别**:
   - depth > 3 的模块适合懒加载
   - 自动生成懒加载建议
6. **架构健康度评估**:
   - 过深的依赖树 = 可能的架构问题
   - 建议架构扁平化
7. **模块分层策略**:
   - 自动划分核心层/业务层/工具层
   - 辅助建立清晰的分层架构

---

## 总结

### 建议实施顺序

1. **dependencies + dependents** (高优先级)
   - 这是最核心的功能，解锁大量高级分析能力
   - 虽然数据量增加较多，但价值远超成本
   - 建议在第一阶段优先实现

2. **depth** (中优先级)
   - 依赖前提: dependencies 已实现
   - 建议在第二阶段实现

### 性能对比

| 字段          | 采集开销              | 数据增长 | 传输影响 |
| ------------- | --------------------- | -------- | -------- |
| dependencies  | 10-300ms              | 5-15%    | 中       |
| dependents    | ~0ms (与 deps 共享)   | 5-15%    | 中       |
| depth         | 5-150ms               | < 1%     | 低       |

### 重要提示

Module 依赖关系（dependencies + dependents）是整个分析系统的基础，许多高级功能都依赖于这两个字段。建议作为第一优先级实现。
