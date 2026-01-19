use derive_more::derive::{Deref, Into};
use rspack_core::Compilation;

use crate::Module;
use crate::context::ModuleChunkContext;
use crate::module_type::ModuleType;

#[derive(Debug, Deref, Into)]
pub struct Modules(pub Vec<Module>);

impl<'a> From<&'a mut Compilation> for Modules {
  fn from(compilation: &'a mut Compilation) -> Self {
    // 保留原有实现用于向后兼容
    let context = ModuleChunkContext::from(&*compilation);
    Self::from_with_context(compilation, &context)
  }
}

impl Modules {
  /// 使用预构建的上下文来避免重复遍历 chunk_graph
  ///
  /// 参数:
  /// - compilation: 编译上下文
  /// - context: 预构建的 module ↔ chunk 映射关系
  pub fn from_with_context(compilation: &mut Compilation, context: &ModuleChunkContext) -> Self {
    let module_graph = compilation.get_module_graph();

    let modules = module_graph
      .modules()
      .into_iter()
      .map(|(id, module)| {
        let name = module.readable_identifier(&compilation.options.context);

        let name_for_condition = module
          .name_for_condition()
          .unwrap_or_default()
          .into_string();

        // 识别模块类型
        let module_type = ModuleType::from_path(&name);

        // 判断是否来自 node_modules
        let is_node_module = name.contains("node_modules/");

        // O(1) 查找，不需要再次遍历 chunk_graph
        let chunks = context
          .module_to_chunks
          .get(&id)
          .cloned()
          .unwrap_or_default();

        Module {
          id: id.to_string(),
          name: name.to_string(),
          name_for_condition,
          size: get_module_size(module.as_ref()),
          chunks,
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
