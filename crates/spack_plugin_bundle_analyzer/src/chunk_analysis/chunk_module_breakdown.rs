use crate::{
  Chunk, Module,
  chunk_analysis::{ModuleSizeInfo, NodeModulesBreakdown, PackageBreakdown, SourceBreakdown},
  package_version_resolver::PackageVersionResolver,
};
use std::collections::HashMap;

/// Chunk 模块大小分解分析
#[derive(Debug, Clone)]
pub struct ChunkModuleBreakdown {
  /// Chunk ID
  pub chunk_id: String,
  /// Chunk 总大小
  pub chunk_size: u64,
  /// 源码分解
  pub source: SourceBreakdown,
  /// 三方包分解
  pub node_modules: NodeModulesBreakdown,
}

impl ChunkModuleBreakdown {
  /// 从 Chunk 和 Modules 构建分析
  pub fn from(chunk: &Chunk, modules: &[Module]) -> Self {
    let mut resolver = PackageVersionResolver::new();

    // 1. 找出这个 chunk 包含的所有模块
    let chunk_modules: Vec<&Module> = modules
      .iter()
      .filter(|m| chunk.modules.contains(&m.id))
      .collect();

    // 2. 分离源码模块和三方包模块
    let mut source_modules = Vec::new();
    let mut node_modules_map: HashMap<String, Vec<ModuleSizeInfo>> = HashMap::new();

    for module in chunk_modules {
      let module_info = ModuleSizeInfo {
        module_id: module.id.clone(),
        module_name: module.name.clone(),
        size: module.size,
        module_type: module.module_type.as_str().to_string(),
        concatenated_modules: module.concatenated_modules.clone(),
      };

      if module.is_node_module {
        // 三方包模块：按包名分组
        if let Some(package_info) = resolver.resolve(&module.name_for_condition) {
          node_modules_map
            .entry(package_info.name.clone())
            .or_default()
            .push(module_info);
        }
      } else {
        // 源码模块
        source_modules.push(module_info);
      }
    }

    // 3. 构建源码分解（按大小降序）
    source_modules.sort_by_key(|m| std::cmp::Reverse(m.size));
    let source_total_size: u64 = source_modules.iter().map(|m| m.size).sum();
    let source = SourceBreakdown {
      total_size: source_total_size,
      module_count: source_modules.len(),
      modules: source_modules,
    };

    // 4. 构建三方包分解
    let mut packages: Vec<PackageBreakdown> = node_modules_map
      .into_iter()
      .map(|(package_name, mut modules)| {
        // 每个包内的模块按大小降序
        modules.sort_by_key(|m| std::cmp::Reverse(m.size));
        let total_size: u64 = modules.iter().map(|m| m.size).sum();

        PackageBreakdown {
          package_name,
          total_size,
          module_count: modules.len(),
          modules,
        }
      })
      .collect();

    // packages 按 total_size 降序
    packages.sort_by_key(|p| std::cmp::Reverse(p.total_size));

    let node_modules_total_size: u64 = packages.iter().map(|p| p.total_size).sum();
    let node_modules = NodeModulesBreakdown {
      total_size: node_modules_total_size,
      package_count: packages.len(),
      packages,
    };

    Self {
      chunk_id: chunk.id.clone(),
      chunk_size: chunk.size,
      source,
      node_modules,
    }
  }
}
