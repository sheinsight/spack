#[derive(Debug)]
pub struct Summary {
  // 总大小（字节）- 原始大小
  pub total_size: u64,
  // gzip 压缩后总大小（字节）
  pub total_gzip_size: u64,
  // brotli 压缩后总大小（字节）
  pub total_brotli_size: u64,
  // 输出文件数量
  pub total_assets: usize,
  // 模块数量
  pub total_modules: usize,
  // chunk 数量
  pub total_chunks: usize,
  // 构建耗时（毫秒）
  pub build_time: f64,
}
