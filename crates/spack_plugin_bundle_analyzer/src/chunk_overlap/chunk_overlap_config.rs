/// Chunk 重叠分析配置
#[derive(Debug, Clone)]
pub struct ChunkOverlapConfig {
  /// 最小模块大小阈值（小于此不报告）
  pub min_module_size: u64,
  /// 最小重复次数（少于此不报告）
  pub min_duplication_count: usize,
  /// 最小浪费空间阈值（小于此不报告）
  pub min_wasted_size: u64,
  /// Chunk 对重叠比例阈值（低于此不报告）
  pub min_overlap_ratio: f64,
  /// 是否包含内部模块（非 node_modules）
  pub include_internal_modules: bool,
}

impl Default for ChunkOverlapConfig {
  fn default() -> Self {
    Self {
      // 1KB - 太小的模块不值得优化
      min_module_size: 1024,
      // 至少重复 2 次
      min_duplication_count: 2,
      // 浪费至少 10KB 才报告
      min_wasted_size: 10 * 1024,
      // chunk 对重叠至少 10%
      min_overlap_ratio: 0.1,
      // 包含内部模块
      include_internal_modules: true,
    }
  }
}
