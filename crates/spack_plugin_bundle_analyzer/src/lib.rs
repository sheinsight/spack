use std::collections::HashMap;

use derive_more::Debug;
use napi::tokio::time::Instant;
use rspack_collections::DatabaseItem;
use rspack_core::{
  ApplyContext, Compilation, CompilerAfterEmit, CompilerOptions, Module, Plugin, PluginContext,
  SourceType,
};
use rspack_hook::{plugin, plugin_hook};
use serde::Serialize;

#[derive(Debug)]
pub struct BundleAnalyzerPluginOpts {
  // pub on_analyzed: Option<CompilationHookFn>,
}

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

  let mut stats = BundleStats {
    modules: Vec::new(),
    total_size: 0,
    chunks: HashMap::new(),
  };

  for (_id, asset) in compilation.assets() {
    let size = asset.source.as_ref().map(|s| s.size()).unwrap_or(0);
    stats.total_size += size as u64;
  }

  let module_graph = compilation.get_module_graph();

  for chunk in compilation.chunk_by_ukey.values() {
    let chunk_name = chunk.name().unwrap_or_default();
    // 获取该 chunk 的所有模块
    let chunk_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
      &chunk.ukey(),
      SourceType::JavaScript,
      &module_graph,
    );

    for module in chunk_modules {
      if let Some(context_module) = module.as_context_module() {
        println!("context_module: {:?}", context_module.get_resolve_options());
      }
    }

    // println!("chunk_modules: {:#?}", chunk_modules);
    // let mut module_list = Vec::new();
    // for module in chunk_modules {
    //   let module_info = ModuleInfo {
    //     name: module.identifier().as_str().to_string(),
    //     size: module.size(&compilation, None) as u64,
    //     path: module.filename().to_string(),
    //     dependencies: module
    //       .dependencies(&compilation, None)
    //       .iter()
    //       .map(|d| d.identifier().as_str().to_string())
    //       .collect(),
    //   };
    // module_list.push(module_info);
    // }
  }

  // 3. 生成分析报告
  // - 文件大小统计
  // - 模块依赖图
  // - 重复模块检测
  // - 分块(chunk)信息等

  let duration = start_time.elapsed().as_secs_f64();
  // let resp = JsBundleAnalyzerPluginResp {
  //   modules: stats.modules,
  //   total_size: stats.total_size,
  //   chunks: stats.chunks,
  //   duration,
  // };

  println!("duration: {:?}", duration);

  Ok(())
}
