/// 模块大小信息
#[derive(Debug, Clone)]
pub struct ModuleSizeInfo {
  /// 模块 ID
  pub module_id: String,
  /// 模块可读名称
  pub module_name: String,
  /// 模块大小（字节）
  pub size: u64,
  /// 占 chunk 的百分比（0.0 ~ 1.0）
  pub size_ratio: f64,
  /// 包名（如果是 node_modules 中的模块）
  pub package_name: Option<String>,
}
