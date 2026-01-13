pub struct Module {
  // 模块唯一 ID
  id: String,
  // 可读名称，如 "./src/index.js"
  name: String,
  // 模块大小
  size: u64,
  // 包含此模块的 chunks
  chunks: Vec<String>,
}
