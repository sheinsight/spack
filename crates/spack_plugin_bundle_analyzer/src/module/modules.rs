use derive_more::derive::{Deref, Into};
use rspack_core::{
  ConcatenatedModule, Compilation, ContextModule, ExternalModule, NormalModule, RawModule,
  SelfModule,
};

use crate::{ConcatenatedModuleInfo, Module, ModuleKind};
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

        // 识别模块类型（使用 name_for_condition 而不是 name，避免 loader 信息干扰）
        let module_type = ModuleType::from_path(&name_for_condition);

        // 判断是否来自 node_modules
        let is_node_module = name.contains("node_modules/");

        // O(1) 查找，不需要再次遍历 chunk_graph
        let chunks = context
          .module_to_chunks
          .get(&id)
          .cloned()
          .unwrap_or_default();

        // 判断模块种类
        let module_kind = get_module_kind(module.as_ref());

        // 尝试 downcast 到 ConcatenatedModule 以获取合并模块信息
        let concatenated_modules = module
          .as_any()
          .downcast_ref::<ConcatenatedModule>()
          .map(|concat_mod| {
            concat_mod
              .get_modules()
              .iter()
              .map(|inner| ConcatenatedModuleInfo {
                id: inner.id.to_string(),
                name: inner.shorten_id.clone(),
                size: inner.size as u64,
              })
              .collect()
          });

        Module {
          id: id.to_string(),
          name: name.to_string(),
          name_for_condition,
          size: get_module_size(module.as_ref()),
          chunks,
          module_kind,
          module_type,
          is_node_module,
          concatenated_modules,
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

/// 通过 downcast 判断模块种类
fn get_module_kind(module: &dyn rspack_core::Module) -> ModuleKind {
  let any_module = module.as_any();

  if any_module.downcast_ref::<ConcatenatedModule>().is_some() {
    ModuleKind::Concatenated
  } else if any_module.downcast_ref::<ExternalModule>().is_some() {
    ModuleKind::External
  } else if any_module.downcast_ref::<ContextModule>().is_some() {
    ModuleKind::Context
  } else if any_module.downcast_ref::<RawModule>().is_some() {
    ModuleKind::Raw
  } else if any_module.downcast_ref::<SelfModule>().is_some() {
    ModuleKind::SelfRef
  } else if any_module.downcast_ref::<NormalModule>().is_some() {
    ModuleKind::Normal
  } else {
    // 如果都不匹配，默认为 Normal（这种情况理论上不应该发生）
    ModuleKind::Normal
  }
}
