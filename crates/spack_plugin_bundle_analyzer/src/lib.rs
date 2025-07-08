use std::collections::HashMap;

use derive_more::Debug;
use napi::tokio::time::Instant;
use rspack_core::{
  ApplyContext, Compilation, CompilerAfterEmit, CompilerOptions, Plugin, PluginContext,
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

  let module_graph = compilation.get_module_graph();

  // 1. 分析输出的资源
  for (id, module) in module_graph.modules() {
    if let Some(source) = compilation.assets().get(&module.identifier().to_string()) {
      let size = source.get_source().map(|s| s.size()).unwrap_or(0);
      let name = module.readable_identifier(&compilation.options.context);
      println!("name: {}", name);
      println!("size: {}", size);
      println!("-----");
      // let path = module.resource().unwrap_or_default().to_string();
      // let dependencies = module
      //   .dependencies()
      //   .iter()
      //   .map(|d| d.identifier().to_string())
      //   .collect();
      // stats.modules.push(ModuleInfo {
      //   name: name.to_string(),
      //   size,
      //   path,
      //   dependencies,
      // });
      // stats.total_size += size;
    }
  }

  let chunk_graph_entries = compilation.get_chunk_graph_entries();
  for entry in chunk_graph_entries {
    // let chunk = entry.chunk();
    // let modules = chunk.get_modules();
    // let size = chunk.get_size();
    // let name = chunk.get_name();
  }

  // 2. 分析模块依赖关系
  let module_graph = compilation.get_module_graph();

  // 3. 生成分析报告
  // - 文件大小统计
  // - 模块依赖图
  // - 重复模块检测
  // - 分块(chunk)信息等

  Ok(())
}
