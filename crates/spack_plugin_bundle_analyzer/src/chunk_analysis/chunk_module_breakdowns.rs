use derive_more::derive::{Deref, Into};

use crate::{Chunk, Module, chunk_analysis::ChunkModuleBreakdown};

#[derive(Debug, Deref, Into)]
pub struct ChunkModuleBreakdowns(pub Vec<ChunkModuleBreakdown>);

impl ChunkModuleBreakdowns {
  /// 为所有 chunks 生成模块分解分析
  pub fn from(chunks: &[Chunk], modules: &[Module]) -> Self {
    Self::from_with_top_n(chunks, modules, 10)
  }

  /// 为所有 chunks 生成模块分解分析，指定 top N
  pub fn from_with_top_n(chunks: &[Chunk], modules: &[Module], top_n: usize) -> Self {
    let breakdowns = chunks
      .iter()
      .map(|chunk| ChunkModuleBreakdown::from_with_top_n(chunk, modules, top_n))
      .collect();

    Self(breakdowns)
  }
}
