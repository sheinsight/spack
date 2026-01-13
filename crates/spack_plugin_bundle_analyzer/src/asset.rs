#[derive(Debug)]
pub struct Asset {
  // 文件名，如 "main.js"
  pub name: String,
  // 文件大小
  pub size: usize,
  // 关联的 chunk
  pub chunks: Vec<String>,
  // 是否实际输出
  pub emitted: bool,
}
