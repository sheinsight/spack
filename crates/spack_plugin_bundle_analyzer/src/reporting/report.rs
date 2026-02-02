use crate::{asset::Asset, chunk::Chunk, module::Module, package::Package};

use super::Summary;

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
}
