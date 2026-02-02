use super::PerformanceTimings;

#[derive(Debug, serde::Serialize)]
pub struct Summary {
  // 总大小（字节）- 原始大小
  pub total_size: u64,
  // gzip 压缩后总大小（字节）
  pub total_gzip_size: u64,
  // 输出文件数量
  pub total_assets: usize,
  // 模块数量
  pub total_modules: usize,
  // chunk 数量
  pub total_chunks: usize,
  // 构建耗时（毫秒）- 已废弃，使用 timings.total_ms
  pub build_time: f64,
  // 详细性能指标
  pub timings: PerformanceTimings,
}
