#![feature(let_chains)]

use std::{
  collections::{HashMap, HashSet},
  fs,
};

use derive_more::Debug;
use napi::tokio::time::Instant;
use rspack_core::{
  ApplyContext, Chunk, ChunkGroupByUkey, ChunkUkey, Compilation, CompilerAfterEmit,
  CompilerOptions, EntrypointsStatsOption, ExtendedStatsOptions, ModuleGraph, ModuleIdentifier,
  Plugin, PluginContext, SourceType,
};
use rspack_hook::{plugin, plugin_hook};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct ModuleReasonInfo {
  pub module_identifier: Option<ModuleIdentifier>,
  pub module_name: Option<String>,
  pub dependency_type: Option<String>,
  pub user_request: Option<String>,
  pub active: bool,
  pub location: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChunkReason {
  module: Option<String>,      // 来源模块
  module_name: Option<String>, // 模块名称
  type_: String,               // 导入类型: "entry", "import", "require", "dynamic import"
  user_request: String,        // 用户请求
  loc: Option<String>,         // 位置信息
}

#[derive(Debug, Serialize)]
struct ChunkAnalysis {
  name: String,
  size: u64,
  initial: bool,
  third_party_packages: HashSet<String>,
  files: HashSet<String>,
  reasons: Vec<ChunkReason>, // 改为 reasons 数组
  origins: Vec<ChunkOrigin>, // 🔍 具体的起源信息
}

#[derive(Debug, Serialize)]
struct ChunkOrigin {
  module: String,            // 模块路径
  module_id: Option<String>, // 改为 String 类型
  location: Option<String>,  // 位置信息
  request: String,           // 导入请求
}

#[derive(Debug)]
pub struct BundleAnalyzerPluginOpts {
  // pub on_analyzed: Option<CompilationHookFn>,
}

#[plugin]
#[derive(Debug)]
pub struct BundleAnalyzerPlugin {
  #[allow(unused)]
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

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &CompilerOptions,
  ) -> rspack_error::Result<()> {
    println!("BundleAnalyzerPlugin apply");
    ctx
      .context
      .compiler_hooks
      .after_emit
      .tap(after_emit::new(self));
    Ok(())
  }
}

#[derive(Debug, Serialize)]
struct ModuleInfo {
  name: String,
  size: u64,
  path: String,
  dependencies: Vec<String>,
}

#[derive(Debug, Serialize)]
struct BundleStats {
  modules: Vec<ModuleInfo>,
  total_size: u64,
  chunks: HashMap<String, Vec<String>>, // chunk名称 -> 模块列表
}

#[plugin_hook(CompilerAfterEmit for BundleAnalyzerPlugin)]
async fn after_emit(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
  let start_time = Instant::now();

  let stats = compilation.get_stats();

  stats
    .get_chunks(
      &ExtendedStatsOptions {
        chunks: true,
        chunk_modules: true,
        assets: false,
        cached_modules: false,
        chunk_group_auxiliary: false,
        chunk_group_children: false,
        chunk_groups: false,
        chunk_relations: false,
        depth: false,
        entrypoints: EntrypointsStatsOption::Bool(false),
        errors: false,
        hash: false,
        ids: false,
        modules: false,
        module_assets: false,
        nested_modules: false,
        optimization_bailout: false,
        provided_exports: false,
        reasons: false,
        source: false,
        used_exports: false,
        warnings: false,
      },
      |chunks| {
        for chunk in chunks {
          println!("--->> {:#?}", chunk);
        }
      },
    )
    .unwrap();

  println!(
    "BundleAnalyzerPlugin -> duration -> {:?}",
    start_time.elapsed().as_millis()
  );

  Ok(())
}
