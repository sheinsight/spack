use std::{
  collections::{HashMap, HashSet},
  path::PathBuf,
};

use byte_unit::{Byte, Unit, UnitType};
use derive_more::Debug;
use napi::tokio::time::Instant;
use package_json_parser::PackageJsonParser;
use rspack_core::{
  ApplyContext, Compilation, CompilerAfterEmit, CompilerOptions, Plugin, PluginContext, SourceType,
};
use rspack_hook::{plugin, plugin_hook};
use serde::Serialize;
use up_finder::UpFinder;

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

  let chunk_group_by_ukey = &compilation.chunk_group_by_ukey;

  for (ukey, chunk) in compilation.chunk_by_ukey.iter() {
    let chunk_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
      ukey,
      SourceType::JavaScript,
      &module_graph,
    );

    let mut third_party_packages = HashSet::new();

    // 分析chunk中的所有模块，看哪些来自node_modules
    for module in chunk_modules {
      let module_path = module.readable_identifier(&compilation.options.context);

      let cwd = PathBuf::from(module_path.to_string());

      let finder = UpFinder::builder().cwd(cwd).build();

      let package_json = finder.find_up("package.json");

      if let Some(package_json) = package_json.first() {
        let package_json = PackageJsonParser::parse(package_json).unwrap();
        if let Some(name) = package_json.name {
          third_party_packages.insert(name.to_string());
        }
      }
    }

    println!("Third-party packages: {:?}", third_party_packages);

    // println!("chunk_modules: {:?}", chunk_modules);

    let _chunk_name = chunk.name().unwrap_or("None");

    let files = chunk.files();
    println!("files: {:?}", files);

    let initial_chunks = chunk.get_all_initial_chunks(chunk_group_by_ukey);

    for initial_chunk in initial_chunks {
      if let Some(chunk) = compilation.chunk_by_ukey.get(&initial_chunk) {
        let chunk_size: u64 = chunk
          .files()
          .iter()
          .filter_map(|file| compilation.assets().get(file))
          .filter_map(|asset| asset.source.as_ref())
          .map(|source| source.size() as u64)
          .sum();

        let chunk_size = Byte::from_u64(chunk_size).get_appropriate_unit(UnitType::Binary);

        println!("initial_chunk: {:?}, size: {:?}", chunk.name(), chunk_size);
      }
    }

    let async_chunks = chunk.get_all_async_chunks(chunk_group_by_ukey);

    for async_chunk in async_chunks {
      if let Some(chunk) = compilation.chunk_by_ukey.get(&async_chunk) {
        let chunk_size: u64 = chunk
          .files()
          .iter()
          .filter_map(|file| compilation.assets().get(file))
          .filter_map(|asset| asset.source.as_ref())
          .map(|source| source.size() as u64)
          .sum();

        let chunk_size = Byte::from_u64(chunk_size).get_appropriate_unit(UnitType::Binary);

        println!("async_chunk: {:?}, size: {:?}", chunk.name(), chunk_size);
      }
    }

    // 获取该 chunk 的所有模块
    // let chunk_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
    //   &chunk.ukey(),
    //   SourceType::JavaScript,
    //   &module_graph,
    // );

    // for module in chunk_modules {
    //   if let Some(context_module) = module.as_context_module() {
    //     println!("context_module: {:?}", context_module.get_resolve_options());
    //   }
    // }

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

fn extract_package_name_from_path(path: &str) -> Option<String> {
  if let Some(node_modules_pos) = path.find("node_modules/") {
    let after_node_modules = &path[node_modules_pos + 13..];

    if after_node_modules.starts_with('@') {
      // Scoped package: @scope/package
      let parts: Vec<&str> = after_node_modules.splitn(3, '/').collect();
      if parts.len() >= 2 {
        return Some(format!("{}/{}", parts[0], parts[1]));
      }
    } else {
      // Regular package
      let package_name = after_node_modules.split('/').next()?;
      return Some(package_name.to_string());
    }
  }
  None
}
