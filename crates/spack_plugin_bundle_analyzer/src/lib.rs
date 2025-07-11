#![feature(let_chains)]

use std::{
  collections::{HashMap, HashSet},
  fs,
};

use derive_more::Debug;
use napi::tokio::time::Instant;
use rspack_core::{
  ApplyContext, Chunk, ChunkGroupByUkey, ChunkUkey, Compilation, CompilerAfterEmit,
  CompilerOptions, ModuleGraph, ModuleIdentifier, Plugin, PluginContext, SourceType,
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

impl BundleAnalyzerPlugin {
  fn get_module_reasons_for_module(
    &self,
    module_graph: &ModuleGraph,
    module_identifier: &ModuleIdentifier,
  ) -> Vec<ModuleReasonInfo> {
    let mut reasons = Vec::new();

    // 获取模块图中的模块
    if let Some(mgm) = module_graph.module_graph_module_by_identifier(module_identifier) {
      // 遍历传入连接
      for dep_id in mgm.incoming_connections() {
        if let Some(connection) = module_graph.connection_by_dependency_id(dep_id) {
          let reason_info = ModuleReasonInfo {
            module_identifier: connection.original_module_identifier,
            module_name: connection
              .original_module_identifier
              .and_then(|id| module_graph.module_by_identifier(&id))
              .map(|m| m.readable_identifier(&Default::default()).to_string()),
            dependency_type: module_graph
              .dependency_by_id(&connection.dependency_id)
              .and_then(|d| d.as_module_dependency())
              .map(|d| d.dependency_type().as_str().to_string()),
            user_request: module_graph
              .dependency_by_id(&connection.dependency_id)
              .and_then(|d| d.as_module_dependency())
              .map(|d| d.user_request().to_string()),
            active: connection.active,
            location: module_graph
              .dependency_by_id(&connection.dependency_id)
              .and_then(|d| d.loc())
              .map(|l| l.to_string()),
          };
          reasons.push(reason_info);
        }
      }
    }

    reasons
  }

  fn get_chunk_reasons_for_chunk(
    &self,
    ukey: &ChunkUkey,
    compilation: &Compilation,
  ) -> Vec<ChunkReason> {
    let mut reasons = Vec::new();

    let module_graph = compilation.get_module_graph();

    let chunk_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
      ukey,
      SourceType::JavaScript,
      &module_graph,
    );

    for module in chunk_modules {
      let module_path = module.readable_identifier(&compilation.options.context);
      // let module_id = module.identifier().as_str();

      let module_reasons = self.get_module_reasons_for_module(&module_graph, &module.identifier());

      for reason in module_reasons {
        reasons.push(ChunkReason {
          module: Some(module_path.to_string()),
          module_name: Some(module_path.to_string()),
          type_: "import".to_string(),
          user_request: reason.user_request.unwrap_or_default(),
          loc: reason.location,
        });
      }
    }

    reasons
  }

  fn get_chunk_reasons_and_origins(
    &self,
    chunk: &Chunk,
    ukey: &ChunkUkey,
    chunk_group_by_ukey: &ChunkGroupByUkey,
    compilation: &Compilation,
  ) -> (Vec<ChunkReason>, Vec<ChunkOrigin>) {
    let is_initial = chunk.can_be_initial(chunk_group_by_ukey);

    // let mut reasons = Vec::new();
    let mut origins = Vec::new();

    let chunk_reason = self.get_chunk_reasons_for_chunk(ukey, compilation);

    if is_initial {
      // 入口 chunk
      if let Some(name) = chunk.name() {
        // reasons.push(ChunkReason {
        //   module: None,
        //   module_name: None,
        //   type_: "entry".to_string(),
        //   user_request: format!("entry:{}", name),
        //   loc: None,
        // });

        origins.push(ChunkOrigin {
          module: "".to_string(),
          module_id: None,
          location: None,
          request: format!("entry:{}", name),
        });
      }
    } else {
      // 非入口 chunk
      let module_graph = compilation.get_module_graph();
      let chunk_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
        ukey,
        SourceType::JavaScript,
        &module_graph,
      );

      for module in chunk_modules {
        let module_path = module.readable_identifier(&compilation.options.context);
        let module_id = module.identifier().as_str();

        // 根据模块路径推断导入类型
        let import_type = if module_path.contains("node_modules") {
          "require"
        } else {
          "import"
        };

        let user_request = if module_path.contains("node_modules") {
          extract_package_name_from_path(&module_path).unwrap_or_else(|| module_path.to_string())
        } else {
          module_path.to_string()
        };

        // reasons.push(chunk_reason);

        origins.push(ChunkOrigin {
          module: module_path.to_string(),
          module_id: Some(module_id.to_string()),
          location: None,
          request: user_request,
        });
      }
    }

    (chunk_reason, origins)
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

  let chunk_group_by_ukey = &compilation.chunk_group_by_ukey;

  let mut chunk_analysis_list = Vec::new();

  for (ukey, chunk) in compilation.chunk_by_ukey.iter() {
    let is_initial = chunk.can_be_initial(chunk_group_by_ukey);

    let chunk_size: u64 = chunk
      .files()
      .iter()
      .filter_map(|file| compilation.assets().get(file))
      .filter_map(|asset| asset.source.as_ref())
      .map(|source| source.size() as u64)
      .sum();

    // let chunk_size = Byte::from_u64(chunk_size).get_appropriate_unit(UnitType::Binary);

    let real_chunk_filename = chunk
      .files()
      .iter()
      .find(|file| file.ends_with(".js") && !file.ends_with(".map"))
      .cloned()
      .unwrap_or_else(|| "unknown.js".to_string());

    let (reasons, origins) =
      self.get_chunk_reasons_and_origins(chunk, ukey, chunk_group_by_ukey, compilation);

    let chunk_analysis = ChunkAnalysis {
      name: real_chunk_filename,
      size: chunk_size,
      initial: is_initial,
      third_party_packages: HashSet::new(),
      files: chunk.files().iter().cloned().collect(),
      reasons, // 使用 reasons 数组
      origins,
    };

    chunk_analysis_list.push(chunk_analysis);
  }

  let duration = start_time.elapsed().as_millis();

  println!("BundleAnalyzerPlugin -> duration -> {:?}", duration);
  // println!("chunk_analysis_list: {:#?}", chunk_analysis_list);

  fs::write(
    "bundle_analyzer_result.json",
    serde_json::to_string_pretty(&chunk_analysis_list).unwrap(),
  )
  .unwrap();

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
