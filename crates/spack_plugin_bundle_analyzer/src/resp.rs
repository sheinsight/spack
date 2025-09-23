use derive_more::Debug;
use serde::{Deserialize, Serialize};

use crate::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleAnalysisResult {
  pub timestamp: u64,
  pub build_time: f64,
  pub summary: SummaryInfo,
  pub modules: Vec<ModuleInfo>,
  pub chunks: Vec<ChunkInfo>,
  pub dependency_graph: Vec<DependencyNode>,
  pub statistics: StatisticsInfo,
  pub visualization: VisualizationData,
}

impl BundleAnalysisResult {
  pub fn new(
    timestamp: u64,
    build_time: f64,
    summary: SummaryInfo,
    modules: Vec<ModuleInfo>,
    chunks: Vec<ChunkInfo>,
    dependency_graph: Vec<DependencyNode>,
    statistics: StatisticsInfo,
    visualization: VisualizationData,
  ) -> Self {
    Self {
      timestamp,
      build_time,
      summary,
      modules,
      chunks,
      dependency_graph,
      statistics,
      visualization,
    }
  }
}
