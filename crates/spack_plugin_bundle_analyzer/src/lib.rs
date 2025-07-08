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

  for (id, asset) in compilation.assets() {
    println!(
      "name: {:?} , size: {:?}",
      id,
      asset.source.as_ref().map(|s| s.size())
    );
  }

  // 2. 分析模块依赖关系
  let module_graph = compilation.get_module_graph();

  for (id, module) in module_graph.modules() {
    let dependencies = module.get_dependencies();
    println!("module: {:?}", id);
    for dependency in dependencies {
      if let Some(dependency) = module_graph
        .dependency_by_id(dependency)
        .and_then(|dep| dep.as_module_dependency())
      {
        println!(" dependency: {:?}", dependency.user_request());
      }
    }
    println!("-----");
    // println!("module: {:?}, dependencies: {:?}", id, dependencies);
  }

  // 3. 生成分析报告
  // - 文件大小统计
  // - 模块依赖图
  // - 重复模块检测
  // - 分块(chunk)信息等

  Ok(())
}
