use crate::ConcatenatedModuleInfo;

/// 模块大小信息
#[derive(Debug, Clone)]
pub struct ModuleSizeInfo {
  /// 模块 ID
  pub module_id: String,
  /// 模块可读名称
  pub module_name: String,
  /// 模块大小（字节）
  pub size: u64,
  /// 模块类型（如 "javascript", "css", "json" 等）
  pub module_type: String,
  /// 合并的模块列表（如果这是一个 ConcatenatedModule）
  pub concatenated_modules: Option<Vec<ConcatenatedModuleInfo>>,
}
