use crate::{asset::Asset, chunk::Chunk, module::Module, summary::Summary};

// 核心数据结构
pub struct Report {
  // 元数据
  timestamp: u64,
  rspack_version: String,
  // 总览
  summary: Summary,

  // 资产（最终输出的文件）
  assets: Vec<Asset>,

  // 模块（源代码文件）
  modules: Vec<Module>,

  // 代码块
  chunks: Vec<Chunk>,
}
