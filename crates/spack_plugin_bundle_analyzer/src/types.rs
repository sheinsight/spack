use std::collections::HashMap;

use derive_more::Debug;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SizeInfo {
  pub original: u64,
  pub minified: u64,
  pub gzipped: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
  pub id: String,
  pub name: String,
  pub path: String,
  pub size: SizeInfo,
  pub module_type: String,
  pub source: String,
  pub is_entry: bool,
  pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
  pub id: String,
  pub name: String,
  pub size: SizeInfo,
  pub modules: Vec<String>,
  pub is_entry: bool,
  pub parents: Vec<String>,
  pub children: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
  pub module_id: String,
  pub dependencies: Vec<DependencyEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
  pub module_id: String,
  pub dependency_type: String,
  pub user_request: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryInfo {
  pub total_modules: usize,
  pub total_chunks: usize,
  pub total_size: SizeInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeStatistics {
  pub count: usize,
  pub total_size: SizeInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceStatistics {
  pub count: usize,
  pub total_size: SizeInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsInfo {
  pub by_file_type: HashMap<String, TypeStatistics>,
  pub by_source: HashMap<String, SourceStatistics>,
  pub largest_modules: Vec<ModuleInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
  pub name: String,
  pub size: u64,
  pub children: Option<Vec<TreeNode>>,
  pub path: Option<String>,
  pub module_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapNode {
  pub name: String,
  pub value: u64,
  pub path: String,
  pub level: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationData {
  pub tree_data: Vec<TreeNode>,
  pub heatmap_data: Vec<HeatmapNode>,
}
