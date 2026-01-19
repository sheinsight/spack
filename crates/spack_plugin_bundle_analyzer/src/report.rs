use crate::{
  asset::Asset,
  chunk::Chunk,
  chunk_analysis::ChunkModuleBreakdown,
  chunk_overlap::ChunkOverlapAnalysis,
  duplicate_packages::DuplicatePackage,
  module::Module,
  package::Package,
  summary::Summary,
};

// 核心数据结构
#[derive(Debug)]
pub struct Report {
  // 元数据
  pub timestamp: u64,
  // 总览
  pub summary: Summary,
  // 资产（最终输出的文件）
  pub assets: Vec<Asset>,
  // 模块（源代码文件）
  pub modules: Vec<Module>,
  // 代码块
  pub chunks: Vec<Chunk>,
  // 包
  pub packages: Vec<Package>,
  // 重复的包
  pub duplicate_packages: Vec<DuplicatePackage>,
  // Chunk 重叠度分析
  pub chunk_overlap: ChunkOverlapAnalysis,
  // Chunk 模块大小分解分析
  pub chunk_module_breakdowns: Vec<ChunkModuleBreakdown>,
}
