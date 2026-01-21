use std::collections::HashMap;

use crate::{
  Chunk, Module,
  chunk_overlap::{
    ChunkOverlapConfig, ChunkPairOverlap, ChunkPairOverlaps, OverlappedModule, OverlappedModules,
  },
};

/// Chunk 重叠度分析报告
#[derive(Debug)]
pub struct ChunkOverlapAnalysis {
  /// 所有重叠的模块（按浪费空间降序）
  pub overlapped_modules: Vec<OverlappedModule>,
  /// Chunk 对之间的重叠关系
  pub chunk_pair_overlaps: Vec<ChunkPairOverlap>,
  /// 总计浪费的空间（字节）
  pub total_wasted_size: u64,
  /// 优化建议
  pub recommendations: Vec<String>,
}

impl ChunkOverlapAnalysis {
  /// 分析 Chunk 重叠度
  pub fn from(chunks: &[Chunk], modules: &[Module]) -> Self {
    let config = ChunkOverlapConfig::default();
    Self::from_with_config(chunks, modules, &config)
  }

  /// 使用自定义配置分析 Chunk 重叠度
  pub fn from_with_config(
    chunks: &[Chunk],
    modules: &[Module],
    config: &ChunkOverlapConfig,
  ) -> Self {
    let mut resolver = crate::package_version_resolver::PackageVersionResolver::new();
    Self::from_with_resolver(chunks, modules, config, &mut resolver)
  }

  /// 使用自定义配置和 resolver 分析 Chunk 重叠度（推荐）
  ///
  /// 参数:
  /// - chunks: chunk 列表
  /// - modules: 模块列表
  /// - config: 重叠度分析配置
  /// - resolver: 可复用的包版本解析器（避免重复创建和缓存失效）
  pub fn from_with_resolver(
    chunks: &[Chunk],
    modules: &[Module],
    config: &ChunkOverlapConfig,
    resolver: &mut crate::package_version_resolver::PackageVersionResolver,
  ) -> Self {
    // 1. 找出重叠的模块
    let overlapped_modules = OverlappedModules::from_with_config(modules, config, resolver);

    // 2. 分析 chunk 对之间的重叠
    let chunk_pair_overlaps = ChunkPairOverlaps::from_with_config(chunks, modules, config);

    // 3. 计算总浪费空间
    let total_wasted_size = overlapped_modules.total_wasted_size();

    // 4. 生成优化建议
    let recommendations = Self::generate_recommendations(&overlapped_modules, &chunk_pair_overlaps);

    Self {
      overlapped_modules: overlapped_modules.into(),
      chunk_pair_overlaps: chunk_pair_overlaps.into(),
      total_wasted_size,
      recommendations,
    }
  }

  /// 生成优化建议
  fn generate_recommendations(
    overlapped_modules: &[OverlappedModule],
    chunk_pair_overlaps: &[ChunkPairOverlap],
  ) -> Vec<String> {
    let mut recommendations = Vec::new();

    // 建议 1: 提取最严重的重复模块
    if let Some(top_module) = overlapped_modules.first() {
      let name = top_module
        .package_name
        .as_ref()
        .unwrap_or(&top_module.module_name);

      recommendations.push(format!(
        "严重重复: {} 出现在 {} 个 chunk 中，浪费 {}KB。建议配置 splitChunks 提取到公共 chunk。",
        name,
        top_module.duplication_count,
        top_module.wasted_size / 1024
      ));
    }

    // 建议 2: 包级别优化
    let mut package_wastage: HashMap<String, u64> = HashMap::new();

    for module in overlapped_modules {
      if let Some(pkg) = &module.package_name {
        *package_wastage.entry(pkg.clone()).or_insert(0) += module.wasted_size;
      }
    }

    let mut packages: Vec<_> = package_wastage.into_iter().collect();
    packages.sort_by_key(|(_, size)| std::cmp::Reverse(*size));

    if let Some((pkg, waste)) = packages.first() {
      if *waste > 50_000 {
        recommendations.push(format!(
          "包级优化: {} 在多个 chunk 中重复，总浪费 {}KB。建议提取到独立的 vendor chunk。",
          pkg,
          waste / 1024
        ));
      }
    }

    // 建议 3: chunk 对重叠过高
    for pair in chunk_pair_overlaps.iter().take(3) {
      if pair.overlap_ratio_a > 0.5 || pair.overlap_ratio_b > 0.5 {
        recommendations.push(format!(
          "Chunk 重叠: {} 和 {} 共享 {}KB ({:.0}% / {:.0}%)。考虑提取公共模块或合并这两个 chunk。",
          pair.chunk_a,
          pair.chunk_b,
          pair.shared_size / 1024,
          pair.overlap_ratio_a * 100.0,
          pair.overlap_ratio_b * 100.0
        ));
      }
    }

    recommendations
  }
}
