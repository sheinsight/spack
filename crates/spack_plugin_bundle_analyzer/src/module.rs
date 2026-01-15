use derive_more::derive::{Deref, Into};
use rspack_collections::Identifier;
use rspack_core::{ChunkGraph, Compilation};

use crate::module_type::ModuleType;

#[derive(Debug)]
pub struct Module {
  // 模块唯一 ID
  pub id: String,
  // 可读名称，如 "./src/index.js"
  pub name: String,
  // 模块大小
  pub size: u64,
  // 包含此模块的 chunks
  pub chunks: Vec<String>,
  // 模块类型
  pub module_type: ModuleType,
  // 是否来自 node_modules
  pub is_node_module: bool,
  // 模块条件名称,用于模块解析条件判断(如 package.json 的 exports 字段)
  pub name_for_condition: String,
}

#[derive(Debug, Deref, Into)]
pub struct Modules(pub Vec<Module>);

impl<'a> From<&'a mut Compilation> for Modules {
  fn from(compilation: &'a mut Compilation) -> Self {
    let module_graph = compilation.get_module_graph();
    let chunk_graph = &compilation.chunk_graph;

    let modules = module_graph
      .modules()
      .into_iter()
      .map(|(id, module)| {
        let name = module
          .readable_identifier(&compilation.options.context)
          .to_string();

        let name_for_condition = module
          .name_for_condition()
          .unwrap_or_default()
          .into_string();

        // 识别模块类型
        let module_type = ModuleType::from_path(&name);

        // 判断是否来自 node_modules
        let is_node_module = name.contains("node_modules/");

        Module {
          id: id.to_string(),
          name,
          name_for_condition,
          size: get_module_size(module.as_ref()),
          chunks: get_module_chunks(&id, chunk_graph),
          module_type,
          is_node_module,
        }
      })
      .collect();
    Modules(modules)
  }
}

fn get_module_size(module: &dyn rspack_core::Module) -> u64 {
  // 使用 Module trait 的 size 方法获取模块大小
  // source_type 参数为 None 表示获取所有类型的总大小
  // compilation 参数为 None 因为我们不需要编译上下文
  module.size(None, None) as u64
}

fn get_module_chunks(module_id: &Identifier, chunk_graph: &ChunkGraph) -> Vec<String> {
  chunk_graph
    .get_module_chunks(*module_id)
    .iter()
    .map(|chunk_ukey| chunk_ukey.as_u32().to_string())
    .collect()
}
