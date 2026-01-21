use derive_more::derive::{Deref, Into};

use crate::{
  Module,
  chunk_overlap::{ChunkOverlapConfig, OverlappedModule},
  package_version_resolver::PackageVersionResolver,
};

#[derive(Debug, Deref, Into)]
pub struct OverlappedModules(pub Vec<OverlappedModule>);

impl OverlappedModules {
  /// 从模块列表中找出重叠的模块
  pub fn from(modules: &[Module]) -> Self {
    let config = ChunkOverlapConfig::default();
    let mut resolver = PackageVersionResolver::new();
    Self::from_with_config(modules, &config, &mut resolver)
  }

  /// 使用自定义配置找出重叠的模块
  ///
  /// 参数:
  /// - modules: 模块列表
  /// - config: 重叠度分析配置
  /// - resolver: 可复用的包版本解析器（避免重复创建和缓存失效）
  pub fn from_with_config(
    modules: &[Module],
    config: &ChunkOverlapConfig,
    resolver: &mut PackageVersionResolver,
  ) -> Self {
    let mut overlapped = Vec::new();

    for module in modules {
      // 至少在 2 个 chunk 中
      if module.chunks.len() < config.min_duplication_count {
        continue;
      }

      // 检查模块大小阈值
      if module.size < config.min_module_size {
        continue;
      }

      // 如果不包含内部模块，跳过非 node_modules 模块
      if !config.include_internal_modules && !module.is_node_module {
        continue;
      }

      // 使用 PackageVersionResolver 获取包名
      // 注意：使用 name_for_condition（真实文件路径）而不是 name（可读标识符）
      let package_name = if module.is_node_module {
        resolver
          .resolve(&module.name_for_condition)
          .map(|info| info.name)
      } else {
        None
      };

      let duplication_count = module.chunks.len();

      // 浪费的空间 = 模块大小 * (重复次数 - 1)
      let wasted_size = module.size * (duplication_count as u64 - 1);

      // 检查浪费空间阈值
      if wasted_size < config.min_wasted_size {
        continue;
      }

      overlapped.push(OverlappedModule {
        module_id: module.id.clone(),
        module_name: module.name.clone(),
        module_size: module.size,
        chunks: module.chunks.clone(),
        duplication_count,
        wasted_size,
        package_name,
      });
    }

    // 按浪费空间降序排序
    overlapped.sort_by_key(|m| std::cmp::Reverse(m.wasted_size));

    Self(overlapped)
  }

  /// 计算总浪费空间
  pub fn total_wasted_size(&self) -> u64 {
    self.iter().map(|m| m.wasted_size).sum()
  }
}
