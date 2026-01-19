use crate::chunk_analysis::ModuleSizeInfo;

/// 源码模块分解
#[derive(Debug, Clone)]
pub struct SourceBreakdown {
  /// 源码总大小
  pub total_size: u64,
  /// 源码模块数量
  pub module_count: usize,
  /// 源码模块列表（按大小降序）
  pub modules: Vec<ModuleSizeInfo>,
}
