use crate::chunk_analysis::ModuleSizeInfo;

/// 单个包的模块分解
#[derive(Debug, Clone)]
pub struct PackageBreakdown {
  /// 包名
  pub package_name: String,
  /// 该包的总大小
  pub total_size: u64,
  /// 该包的模块数量
  pub module_count: usize,
  /// 该包的模块列表（按大小降序）
  pub modules: Vec<ModuleSizeInfo>,
}
