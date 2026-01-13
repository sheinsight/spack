pub struct Chunk {
  // chunk ID
  id: String,
  // chunk 名称
  names: Vec<String>,
  // chunk 大小
  size: u64,
  // 包含的模块 ID 列表
  modules: Vec<String>,
  // 是否入口 chunk
  entry: bool,
  // 是否初始 chunk
  initial: bool,
}
