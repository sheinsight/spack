#[derive(Debug)]
pub struct Asset {
  // 文件名，如 "main.js"
  pub name: String,
  // 文件大小（原始大小）
  pub size: usize,
  // gzip 压缩后大小
  pub gzip_size: Option<usize>,
  // brotli 压缩后大小
  pub brotli_size: Option<usize>,
  // 关联的 chunk
  pub chunks: Vec<String>,
  // 是否实际输出
  pub emitted: bool,
}
