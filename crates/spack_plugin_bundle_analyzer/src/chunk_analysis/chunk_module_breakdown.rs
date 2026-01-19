use crate::{
  Chunk, Module, chunk_analysis::ModuleSizeInfo, package_version_resolver::PackageVersionResolver,
};

/// Chunk 模块大小分解分析
#[derive(Debug, Clone)]
pub struct ChunkModuleBreakdown {
  /// Chunk ID
  pub chunk_id: String,
  /// Chunk 总大小
  pub chunk_size: u64,
  /// 所有模块信息（按大小降序）
  pub modules: Vec<ModuleSizeInfo>,
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

    // 2. 转换为 ModuleSizeInfo（带大小、占比）
    let mut modules_info: Vec<ModuleSizeInfo> = chunk_modules
      .iter()
      .map(|m| {
        // 使用 PackageVersionResolver 获取包名
        let package_name = if m.is_node_module {
          resolver
            .resolve(&m.name_for_condition)
            .map(|info| info.name)
        } else {
          None
        };

        ModuleSizeInfo {
          module_id: m.id.clone(),
          module_name: m.name.clone(),
          size: m.size,
          size_ratio: if chunk.size > 0 {
            m.size as f64 / chunk.size as f64
          } else {
            0.0
          },
          package_name,
        }
      })
      .collect();

    // 3. 按大小降序排序
    modules_info.sort_by_key(|m| std::cmp::Reverse(m.size));

    Self {
      chunk_id: chunk.id.clone(),
      chunk_size: chunk.size,
      modules: modules_info,
    }
  }
}
