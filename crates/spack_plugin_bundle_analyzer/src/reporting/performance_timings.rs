/// 性能计时信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceTimings {
  /// 收集 Assets 耗时（毫秒）
  pub collect_assets_ms: f64,
  /// 收集 Modules 耗时（毫秒）
  pub collect_modules_ms: f64,
  /// 收集 Chunks 耗时（毫秒）
  pub collect_chunks_ms: f64,
  /// 分析 Packages 耗时（毫秒）
  pub analyze_packages_ms: f64,
  /// 总耗时（毫秒）
  pub total_ms: f64,
}

impl PerformanceTimings {
  pub fn new(
    collect_assets_ms: f64,
    collect_modules_ms: f64,
    collect_chunks_ms: f64,
    analyze_packages_ms: f64,
    total_ms: f64,
  ) -> Self {
    Self {
      collect_assets_ms,
      collect_modules_ms,
      collect_chunks_ms,
      analyze_packages_ms,
      total_ms,
    }
  }
}
