#![feature(let_chains)]

mod opts;
mod resp;
mod types;

pub use opts::{BundleAnalyzerPluginOpts, CompilationHookFn};
pub use resp::*;
pub use types::*;

use derive_more::Debug;
use napi::tokio::time::Instant;
use rspack_core::{
  ApplyContext, ChunkUkey, Compilation, CompilerAfterEmit, ModuleIdentifier, Plugin,
};
use rspack_hook::{plugin, plugin_hook};
use std::collections::HashMap;

#[plugin]
#[derive(Debug)]
pub struct BundleAnalyzerPlugin {
  options: BundleAnalyzerPluginOpts,
}

impl BundleAnalyzerPlugin {
  pub fn new(options: BundleAnalyzerPluginOpts) -> Self {
    Self::new_inner(options)
  }
}

impl Plugin for BundleAnalyzerPlugin {
  fn name(&self) -> &'static str {
    "spack.BundleAnalyzerPlugin"
  }

  fn apply(&self, ctx: &mut ApplyContext) -> rspack_error::Result<()> {
    ctx
      .compiler_hooks
      .after_emit
      .tap(after_emit::new(self));
    Ok(())
  }
}

#[plugin_hook(CompilerAfterEmit for BundleAnalyzerPlugin)]
async fn after_emit(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
  let start_time = Instant::now();
  
  let analyzer_result = analyze_bundle(compilation).await;
  let duration = start_time.elapsed().as_millis() as f64;
  
  let response = BundleAnalysisResult {
    timestamp: std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_millis() as u64,
    build_time: duration,
    summary: analyzer_result.summary,
    modules: analyzer_result.modules,
    chunks: analyzer_result.chunks,
    dependency_graph: analyzer_result.dependency_graph,
    statistics: analyzer_result.statistics,
    visualization: analyzer_result.visualization,
  };

  if let Some(on_analyzed) = &self.options.on_analyzed {
    if let Err(e) = on_analyzed(response).await {
      println!("bundle-analyzer-plugin-error: {:?}", e);
    }
  }

  Ok(())
}

async fn analyze_bundle(compilation: &Compilation) -> BundleAnalysisResult {
  let module_graph = compilation.get_module_graph();
  let chunk_graph = &compilation.chunk_graph;
  
  // 收集模块信息
  let mut modules = Vec::new();
  let mut module_map = HashMap::new();
  
  for (module_id, module) in module_graph.modules() {
    let readable_name = module.readable_identifier(&compilation.options.context);
    let module_type = get_module_type(&readable_name);
    let source_type = get_source_type(&readable_name);
    
    // 获取模块大小
    let size = get_module_size(module.as_ref(), compilation);
    
    // 获取依赖
    let dependencies = get_module_dependencies(&module_id, &module_graph);
    
    let module_info = ModuleInfo {
      id: module_id.to_string(),
      name: readable_name.to_string(),
      path: readable_name.to_string(),
      size: SizeInfo {
        original: size,
        minified: size, // 简化处理，实际项目中需要更精确的计算
        gzipped: (size as f64 * 0.3) as u64, // 估算 gzip 压缩比
      },
      module_type,
      source: source_type,
      is_entry: check_is_entry_module(&module_id, compilation),
      dependencies,
    };
    
    modules.push(module_info.clone());
    module_map.insert(module_id.to_string(), module_info);
  }
  
  // 收集代码块信息
  let mut chunks = Vec::new();
  let chunk_by_ukey = compilation.chunk_by_ukey.clone();
  
  for (chunk_ukey, chunk) in chunk_by_ukey.iter() {
    let chunk_modules = chunk_graph
      .get_chunk_modules(chunk_ukey, &module_graph)
      .into_iter()
      .map(|m| m.identifier().to_string())
      .collect::<Vec<_>>();
    
    let total_size = chunk_modules
      .iter()
      .filter_map(|id| module_map.get(id))
      .fold(SizeInfo::default(), |mut acc, module| {
        acc.original += module.size.original;
        acc.minified += module.size.minified;
        acc.gzipped += module.size.gzipped;
        acc
      });
    
    let chunk_info = ChunkInfo {
      id: chunk_ukey.as_u32().to_string(),
      name: chunk.name().map(|s| s.to_string()).unwrap_or_else(|| format!("chunk-{}", chunk_ukey.as_u32())),
      size: total_size,
      modules: chunk_modules,
      is_entry: chunk.has_entry_module(&chunk_graph),
      parents: get_chunk_parents(chunk_ukey, compilation),
      children: get_chunk_children(chunk_ukey, compilation),
    };
    
    chunks.push(chunk_info);
  }
  
  // 构建依赖关系图
  let dependency_graph = build_dependency_graph(&modules);
  
  // 计算统计信息
  let statistics = calculate_statistics(&modules);
  
  // 生成可视化数据
  let visualization = generate_visualization_data(&modules, &chunks);
  
  // 计算摘要信息
  let total_size = modules.iter().fold(SizeInfo::default(), |mut acc, module| {
    acc.original += module.size.original;
    acc.minified += module.size.minified;
    acc.gzipped += module.size.gzipped;
    acc
  });
  
  let summary = SummaryInfo {
    total_modules: modules.len(),
    total_chunks: chunks.len(),
    total_size,
  };
  
  BundleAnalysisResult {
    timestamp: 0, // 将在调用处设置
    build_time: 0.0, // 将在调用处设置
    summary,
    modules,
    chunks,
    dependency_graph,
    statistics,
    visualization,
  }
}

fn get_module_type(path: &str) -> String {
  if path.ends_with(".js") || path.ends_with(".ts") || path.ends_with(".jsx") || path.ends_with(".tsx") {
    "javascript".to_string()
  } else if path.ends_with(".css") || path.ends_with(".scss") || path.ends_with(".sass") {
    "css".to_string()
  } else if path.ends_with(".png") || path.ends_with(".jpg") || path.ends_with(".jpeg") || path.ends_with(".gif") || path.ends_with(".svg") {
    "image".to_string()
  } else {
    "other".to_string()
  }
}

fn get_source_type(path: &str) -> String {
  if path.contains("node_modules") {
    "node_modules".to_string()
  } else if path.contains("src") {
    "src".to_string()
  } else {
    "other".to_string()
  }
}

fn get_module_size(module: &dyn rspack_core::Module, _compilation: &Compilation) -> u64 {
  // 简化实现，基于模块标识符长度估算大小
  module.identifier().to_string().len() as u64 * 100
}

fn get_module_dependencies(_module_id: &ModuleIdentifier, _module_graph: &rspack_core::ModuleGraph) -> Vec<String> {
  // 简化实现，暂时返回空依赖列表
  // 实际项目中需要从 module_graph 中获取真实的依赖关系
  Vec::new()
}

fn check_is_entry_module(module_id: &ModuleIdentifier, compilation: &Compilation) -> bool {
  compilation.entries.values().any(|entry| {
    entry.dependencies.iter().any(|dep| {
      compilation.get_module_graph()
        .get_module_by_dependency_id(dep)
        .map_or(false, |mid| mid.identifier() == *module_id)
    })
  })
}

fn get_chunk_parents(_chunk_ukey: &ChunkUkey, _compilation: &Compilation) -> Vec<String> {
  // 简化实现，返回空列表
  // 实际项目中需要从 chunk_graph 中获取父子关系
  Vec::new()
}

fn get_chunk_children(_chunk_ukey: &ChunkUkey, _compilation: &Compilation) -> Vec<String> {
  // 简化实现，返回空列表
  // 实际项目中需要从 chunk_graph 中获取父子关系
  Vec::new()
}

fn build_dependency_graph(modules: &[ModuleInfo]) -> Vec<DependencyNode> {
  modules
    .iter()
    .map(|module| DependencyNode {
      module_id: module.id.clone(),
      dependencies: module
        .dependencies
        .iter()
        .map(|dep_id| DependencyEdge {
          module_id: dep_id.clone(),
          dependency_type: "import".to_string(), // 简化处理
          user_request: dep_id.clone(),
        })
        .collect(),
    })
    .collect()
}

fn calculate_statistics(modules: &[ModuleInfo]) -> StatisticsInfo {
  let mut by_file_type = HashMap::new();
  let mut by_source = HashMap::new();
  
  for module in modules {
    // 按文件类型分组
    let type_stats = by_file_type.entry(module.module_type.clone()).or_insert(TypeStatistics {
      count: 0,
      total_size: SizeInfo::default(),
    });
    type_stats.count += 1;
    type_stats.total_size.original += module.size.original;
    type_stats.total_size.minified += module.size.minified;
    type_stats.total_size.gzipped += module.size.gzipped;
    
    // 按来源分组
    let source_stats = by_source.entry(module.source.clone()).or_insert(SourceStatistics {
      count: 0,
      total_size: SizeInfo::default(),
    });
    source_stats.count += 1;
    source_stats.total_size.original += module.size.original;
    source_stats.total_size.minified += module.size.minified;
    source_stats.total_size.gzipped += module.size.gzipped;
  }
  
  // 找出最大的10个模块
  let mut largest_modules = modules.to_vec();
  largest_modules.sort_by_key(|m| std::cmp::Reverse(m.size.original));
  largest_modules.truncate(10);
  
  StatisticsInfo {
    by_file_type,
    by_source,
    largest_modules,
  }
}

fn generate_visualization_data(modules: &[ModuleInfo], _chunks: &[ChunkInfo]) -> VisualizationData {
  // 生成简化的树形结构数据
  let tree_data = modules
    .iter()
    .map(|module| TreeNode {
      name: module.name.clone(),
      size: module.size.original,
      children: None,
      path: Some(module.path.clone()),
      module_type: Some(module.module_type.clone()),
    })
    .collect();
  
  // 生成热力图数据
  let heatmap_data = modules
    .iter()
    .map(|module| HeatmapNode {
      name: module.name.clone(),
      value: module.size.original,
      path: module.path.clone(),
      level: module.path.split('/').count(),
    })
    .collect();
  
  VisualizationData {
    tree_data,
    heatmap_data,
  }
}