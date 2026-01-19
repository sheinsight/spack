use derive_more::derive::{Deref, Into};

use crate::{Chunk, Module, chunk_analysis::ChunkModuleBreakdown};

#[derive(Debug, Deref, Into)]
pub struct ChunkModuleBreakdowns(pub Vec<ChunkModuleBreakdown>);

impl ChunkModuleBreakdowns {
  /// 为所有 chunks 生成模块分解分析
  pub fn from(chunks: &[Chunk], modules: &[Module]) -> Self {
    let breakdowns = chunks
      .iter()
      .map(|chunk| ChunkModuleBreakdown::from(chunk, modules))
      .collect();

    Self(breakdowns)
  }
}
