pub struct Summary {
  // 总大小（字节）
  total_size: u64,
  // 输出文件数量
  total_assets: usize,
  // 模块数量
  total_modules: usize,
  // chunk 数量
  total_chunks: usize,
  // 构建耗时（毫秒）
  build_time: f64,
}
