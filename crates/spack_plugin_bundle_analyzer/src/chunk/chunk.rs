#[derive(Debug, serde::Serialize)]
pub struct Chunk {
  // chunk ID
  pub id: String,
  // chunk 名称
  pub names: Vec<String>,
  // chunk 大小
  pub size: u64,
  // 包含的模块 ID 列表
  pub modules: Vec<String>,
  // 是否入口 chunk
  pub entry: bool,
  // 是否初始 chunk
  pub initial: bool,
  // chunk 创建的原因(如 entry、import()、splitChunks 等)
  pub reason: String,
  // chunk 生成的输出文件列表
  pub files: Vec<String>,
  // 是否包含异步 chunk
  pub async_chunks: bool,
  // 是否包含运行时代码
  pub runtime: bool,
  // 父 chunk ID 列表（哪些 chunk 引用了当前 chunk）
  pub parents: Vec<String>,
  // 子 chunk ID 列表（当前 chunk 引用了哪些 chunk）
  pub children: Vec<String>,
}
