use std::collections::HashMap;

use derive_more::Debug;
use napi::{Env, Unknown, bindgen_prelude::FromNapiValue};
use napi_derive::napi;
use rspack_core::BoxPlugin;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use spack_macros::ThreadsafeCallback;
use spack_plugin_bundle_analyzer::{
  BundleAnalysisResult, BundleAnalyzerPlugin, BundleAnalyzerPluginOpts, ChunkInfo, DependencyEdge,
  DependencyNode, HeatmapNode, ModuleInfo, SizeInfo, SourceStatistics, StatisticsInfo, SummaryInfo,
  TreeNode, TypeStatistics, VisualizationData,
};

#[derive(Debug, ThreadsafeCallback)]
#[napi(object, object_to_js = false)]
pub struct RawBundleAnalyzerPluginOpts {
  #[napi(ts_type = "(response: JsBundleAnalyzerPluginResp) => void|Promise<void>")]
  #[debug(skip)]
  #[threadsafe_callback]
  pub on_analyzed: Option<ThreadsafeFunction<JsBundleAnalyzerPluginResp, ()>>,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsSizeInfo {
  pub original: f64,
  pub minified: f64,
  pub gzipped: f64,
}

impl From<SizeInfo> for JsSizeInfo {
  fn from(value: SizeInfo) -> Self {
    Self {
      original: value.original as f64,
      minified: value.minified as f64,
      gzipped: value.gzipped as f64,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsModuleInfo {
  pub id: String,
  pub name: String,
  pub path: String,
  pub size: JsSizeInfo,
  #[napi(js_name = "moduleType")]
  pub module_type: String,
  pub source: String,
  #[napi(js_name = "isEntry")]
  pub is_entry: bool,
  pub dependencies: Vec<String>,
}

impl From<ModuleInfo> for JsModuleInfo {
  fn from(value: ModuleInfo) -> Self {
    Self {
      id: value.id,
      name: value.name,
      path: value.path,
      size: value.size.into(),
      module_type: value.module_type,
      source: value.source,
      is_entry: value.is_entry,
      dependencies: value.dependencies,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsChunkInfo {
  pub id: String,
  pub name: String,
  pub size: JsSizeInfo,
  pub modules: Vec<String>,
  #[napi(js_name = "isEntry")]
  pub is_entry: bool,
  pub parents: Vec<String>,
  pub children: Vec<String>,
}

impl From<ChunkInfo> for JsChunkInfo {
  fn from(value: ChunkInfo) -> Self {
    Self {
      id: value.id,
      name: value.name,
      size: value.size.into(),
      modules: value.modules,
      is_entry: value.is_entry,
      parents: value.parents,
      children: value.children,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsSummaryInfo {
  #[napi(js_name = "totalModules")]
  pub total_modules: f64,
  #[napi(js_name = "totalChunks")]
  pub total_chunks: f64,
  #[napi(js_name = "totalSize")]
  pub total_size: JsSizeInfo,
}

impl From<SummaryInfo> for JsSummaryInfo {
  fn from(value: SummaryInfo) -> Self {
    Self {
      total_modules: value.total_modules as f64,
      total_chunks: value.total_chunks as f64,
      total_size: value.total_size.into(),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsTypeStatistics {
  pub count: f64,
  #[napi(js_name = "totalSize")]
  pub total_size: JsSizeInfo,
}

impl From<TypeStatistics> for JsTypeStatistics {
  fn from(value: TypeStatistics) -> Self {
    Self {
      count: value.count as f64,
      total_size: value.total_size.into(),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsSourceStatistics {
  pub count: f64,
  #[napi(js_name = "totalSize")]
  pub total_size: JsSizeInfo,
}

impl From<SourceStatistics> for JsSourceStatistics {
  fn from(value: SourceStatistics) -> Self {
    Self {
      count: value.count as f64,
      total_size: value.total_size.into(),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsDependencyEdge {
  #[napi(js_name = "moduleId")]
  pub module_id: String,
  #[napi(js_name = "dependencyType")]
  pub dependency_type: String,
  #[napi(js_name = "userRequest")]
  pub user_request: String,
}

impl From<DependencyEdge> for JsDependencyEdge {
  fn from(value: DependencyEdge) -> Self {
    Self {
      module_id: value.module_id,
      dependency_type: value.dependency_type,
      user_request: value.user_request,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsDependencyNode {
  #[napi(js_name = "moduleId")]
  pub module_id: String,
  pub dependencies: Vec<JsDependencyEdge>,
}

impl From<DependencyNode> for JsDependencyNode {
  fn from(value: DependencyNode) -> Self {
    Self {
      module_id: value.module_id,
      dependencies: value.dependencies.into_iter().map(Into::into).collect(),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsTreeNode {
  pub name: String,
  pub size: f64,
  pub children: Option<Vec<JsTreeNode>>,
  pub path: Option<String>,
  #[napi(js_name = "moduleType")]
  pub module_type: Option<String>,
}

impl From<TreeNode> for JsTreeNode {
  fn from(value: TreeNode) -> Self {
    Self {
      name: value.name,
      size: value.size as f64,
      children: value
        .children
        .map(|children| children.into_iter().map(Into::into).collect()),
      path: value.path,
      module_type: value.module_type,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsHeatmapNode {
  pub name: String,
  pub value: f64,
  pub path: String,
  pub level: f64,
}

impl From<HeatmapNode> for JsHeatmapNode {
  fn from(value: HeatmapNode) -> Self {
    Self {
      name: value.name,
      value: value.value as f64,
      path: value.path,
      level: value.level as f64,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsVisualizationData {
  #[napi(js_name = "treeData")]
  pub tree_data: Vec<JsTreeNode>,
  #[napi(js_name = "heatmapData")]
  pub heatmap_data: Vec<JsHeatmapNode>,
}

impl From<VisualizationData> for JsVisualizationData {
  fn from(value: VisualizationData) -> Self {
    Self {
      tree_data: value.tree_data.into_iter().map(Into::into).collect(),
      heatmap_data: value.heatmap_data.into_iter().map(Into::into).collect(),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsStatisticsInfo {
  #[napi(js_name = "byFileType")]
  pub by_file_type: HashMap<String, JsTypeStatistics>,
  #[napi(js_name = "bySource")]
  pub by_source: HashMap<String, JsSourceStatistics>,
  #[napi(js_name = "largestModules")]
  pub largest_modules: Vec<JsModuleInfo>,
}

impl From<StatisticsInfo> for JsStatisticsInfo {
  fn from(value: StatisticsInfo) -> Self {
    Self {
      by_file_type: value
        .by_file_type
        .into_iter()
        .map(|(k, v)| (k, v.into()))
        .collect(),
      by_source: value
        .by_source
        .into_iter()
        .map(|(k, v)| (k, v.into()))
        .collect(),
      largest_modules: value.largest_modules.into_iter().map(Into::into).collect(),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsBundleAnalyzerPluginResp {
  pub timestamp: f64,
  #[napi(js_name = "buildTime")]
  pub build_time: f64,
  pub summary: JsSummaryInfo,
  pub modules: Vec<JsModuleInfo>,
  pub chunks: Vec<JsChunkInfo>,
  #[napi(js_name = "dependencyGraph")]
  pub dependency_graph: Vec<JsDependencyNode>,
  pub statistics: JsStatisticsInfo,
  pub visualization: JsVisualizationData,
}

impl From<BundleAnalysisResult> for JsBundleAnalyzerPluginResp {
  fn from(value: BundleAnalysisResult) -> Self {
    Self {
      timestamp: value.timestamp as f64,
      build_time: value.build_time,
      summary: value.summary.into(),
      modules: value.modules.into_iter().map(Into::into).collect(),
      chunks: value.chunks.into_iter().map(Into::into).collect(),
      dependency_graph: value.dependency_graph.into_iter().map(Into::into).collect(),
      statistics: value.statistics.into(),
      visualization: value.visualization.into(),
    }
  }
}

pub fn binding(_env: Env, options: Unknown<'_>) -> napi::Result<BoxPlugin> {
  let options = RawBundleAnalyzerPluginOpts::from_unknown(options)?;
  Ok(Box::new(BundleAnalyzerPlugin::new(options.into())) as BoxPlugin)
}
