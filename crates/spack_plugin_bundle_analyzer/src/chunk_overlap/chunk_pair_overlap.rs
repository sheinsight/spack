/// Chunk 对之间的重叠信息
#[derive(Debug, Clone)]
pub struct ChunkPairOverlap {
  /// Chunk A 的 ID
  pub chunk_a: String,
  /// Chunk B 的 ID
  pub chunk_b: String,
  /// 共享的模块 ID 列表
  pub shared_modules: Vec<String>,
  /// 共享部分的总大小（字节）
  pub shared_size: u64,
  /// 占 Chunk A 的比例
  pub overlap_ratio_a: f64,
  /// 占 Chunk B 的比例
  pub overlap_ratio_b: f64,
}
