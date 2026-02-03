use std::collections::HashMap;

use derive_more::derive::{Deref, Into};
use rspack_core::{
  Compilation, ConcatenatedModule, ContextModule, ExternalModule, NormalModule, RawModule,
  SelfModule,
};

use super::ModuleType;
use crate::context::ModuleChunkContext;
use crate::package::Packages;
use crate::{ConcatenatedModuleInfo, Module, ModuleKind};

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

        // 一次性判断模块种类并提取 ConcatenatedModule 信息（避免重复 downcast）
        let (module_kind, concatenated_modules) =
          Self::extract_module_kind_and_concat(module.as_ref(), &module_graph, compilation);

        let user_request = module
          .as_normal_module()
          .map(|m| m.user_request().to_string());

        let raw_request = module
          .as_normal_module()
          .map(|m| m.raw_request().to_string());

        Module {
          id: id.to_string(),
          name: name.to_string(),
          name_for_condition,
          user_request,
          raw_request,
          size: get_module_size(module.as_ref()),
          chunks,
          module_kind,
          module_type,
          is_node_module,
          concatenated_modules,
          package_json_path: None, // 初始为 None，后续通过 associate_packages 关联
        }
      })
      .collect();
    Modules(modules)
  }

  /// 判断模块种类并提取 ConcatenatedModule 信息（一次 downcast 完成）
  ///
  /// 性能优化：
  /// - 按出现频率排序检查（Normal 最常见 ~92%，优先检查可早返回）
  /// - 合并了原来的 `get_module_kind()` 和 ConcatenatedModule downcast
  /// - 避免了对 ConcatenatedModule 的重复 downcast（从 2 次减少到 1 次）
  fn extract_module_kind_and_concat(
    module: &dyn rspack_core::Module,
    module_graph: &rspack_core::ModuleGraph,
    compilation: &Compilation,
  ) -> (ModuleKind, Option<Vec<ConcatenatedModuleInfo>>) {
    let any = module.as_any();

    // 按出现频率优化检查顺序（Normal 约 92%，最常见）
    if any.downcast_ref::<NormalModule>().is_some() {
      return (ModuleKind::Normal, None);
    }

    // Concatenated 约 6%，第二常见，且需要提取内部模块信息
    if let Some(concat_mod) = any.downcast_ref::<ConcatenatedModule>() {
      let inner_modules = Self::extract_concatenated_info(concat_mod, module_graph, compilation);
      return (ModuleKind::Concatenated, Some(inner_modules));
    }

    // Context 约 1.5%
    if any.downcast_ref::<ContextModule>().is_some() {
      return (ModuleKind::Context, None);
    }

    // External 约 0.3%
    if any.downcast_ref::<ExternalModule>().is_some() {
      return (ModuleKind::External, None);
    }

    // Raw 约 0.1%
    if any.downcast_ref::<RawModule>().is_some() {
      return (ModuleKind::Raw, None);
    }

    // SelfRef 约 0.1%
    if any.downcast_ref::<SelfModule>().is_some() {
      return (ModuleKind::SelfRef, None);
    }

    // Fallback（理论上不应该发生）
    (ModuleKind::Normal, None)
  }

  /// 提取 ConcatenatedModule 的内部模块信息
  ///
  /// ConcatenatedModule 是 rspack 的 scope hoisting 优化产生的合并模块，
  /// 包含多个原始模块的信息。这个函数提取所有内部模块的详细信息。
  fn extract_concatenated_info(
    concat_mod: &ConcatenatedModule,
    module_graph: &rspack_core::ModuleGraph,
    compilation: &Compilation,
  ) -> Vec<ConcatenatedModuleInfo> {
    concat_mod
      .get_modules()
      .iter()
      .map(|inner| {
        // 尝试在 module_graph 中查找原始模块以获取完整信息
        let (inner_name_for_condition, inner_is_node_module, inner_module_type) =
          if let Some(inner_module) = module_graph.module_by_identifier(&inner.id) {
            let inner_name = inner_module.readable_identifier(&compilation.options.context);
            let inner_name_for_condition = inner_module
              .name_for_condition()
              .unwrap_or_default()
              .into_string();
            let inner_is_node_module = inner_name.contains("node_modules/");
            let inner_module_type = ModuleType::from_path(&inner_name_for_condition);

            (
              inner_name_for_condition,
              inner_is_node_module,
              inner_module_type,
            )
          } else {
            // Fallback: 从 shorten_id 解析（当原始模块信息不可用时）
            let fallback_name = inner.shorten_id.clone();
            let fallback_is_node_module = fallback_name.contains("node_modules/");
            let fallback_module_type = ModuleType::from_path(&fallback_name);

            (
              fallback_name.clone(),
              fallback_is_node_module,
              fallback_module_type,
            )
          };

        ConcatenatedModuleInfo {
          id: inner.id.to_string(),
          name: inner.shorten_id.clone(),
          size: inner.size as u64,
          module_type: inner_module_type,
          is_node_module: inner_is_node_module,
          name_for_condition: inner_name_for_condition,
          package_json_path: None, // 后续通过 associate_packages 填充
        }
      })
      .collect()
  }

  /// 将 Modules 与 Packages 关联
  ///
  /// 通过 Package.modules 建立 module_id → package 的映射，
  /// 为每个来自三方包的 Module 填充 package_json_path 字段，
  /// 同时递归处理 ConcatenatedModuleInfo 中的内部模块
  ///
  /// 参数:
  /// - packages: 已分析的 Packages
  pub fn associate_packages(&mut self, packages: &Packages) {
    // 预计算 HashMap 容量，避免动态扩容和 rehash
    // 容量 = 所有 package 的 modules 数量总和
    let estimated_capacity: usize = packages.iter().map(|p| p.modules.len()).sum();

    // 构建 module_id → package 映射（O(n)）
    let mut module_package_map: HashMap<String, &crate::Package> =
      HashMap::with_capacity(estimated_capacity);

    for package in packages.iter() {
      for module_id in &package.modules {
        module_package_map.insert(module_id.clone(), package);
      }
    }

    // 为每个 Module 填充 package_json_path（O(m)）
    for module in &mut self.0 {
      // 1. 处理外层模块
      if let Some(package) = module_package_map.get(&module.id) {
        module.package_json_path = Some(package.package_json_path.clone());
      }

      // 2. 递归处理内部的 ConcatenatedModuleInfo
      if let Some(ref mut inner_modules) = module.concatenated_modules {
        for inner in inner_modules {
          if let Some(package) = module_package_map.get(&inner.id) {
            inner.package_json_path = Some(package.package_json_path.clone());
          }
        }
      }
    }
  }
}

fn get_module_size(module: &dyn rspack_core::Module) -> u64 {
  // 使用 Module trait 的 size 方法获取模块大小
  // source_type 参数为 None 表示获取所有类型的总大小
  // compilation 参数为 None 因为我们不需要编译上下文
  module.size(None, None) as u64
}
