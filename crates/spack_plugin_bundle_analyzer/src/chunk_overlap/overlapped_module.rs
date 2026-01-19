/// 重叠模块信息
#[derive(Debug, Clone)]
pub struct OverlappedModule {
  /// 模块 ID
  pub module_id: String,
  /// 模块可读名称
  pub module_name: String,
  /// 模块大小（字节）
  pub module_size: u64,
  /// 包含此模块的 chunk IDs
  pub chunks: Vec<String>,
  /// 重复次数（chunks.len()）
  pub duplication_count: usize,
  /// 浪费的空间（size * (count - 1)）
  pub wasted_size: u64,
  /// 包名（如果是 node_modules 中的模块）
  pub package_name: Option<String>,
}
