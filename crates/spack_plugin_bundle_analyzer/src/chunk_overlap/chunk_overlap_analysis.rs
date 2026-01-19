use std::collections::{HashMap, HashSet};

use crate::{
  Chunk, ChunkPairOverlap, Module, OverlappedModule,
  chunk_overlap::{ChunkOverlapConfig, OverlappedModules},
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
    // 构建 module_id -> module 映射
    let module_map: HashMap<String, &Module> = modules.iter().map(|m| (m.id.clone(), m)).collect();

    // 1. 找出重叠的模块
    let overlapped_modules = OverlappedModules::from_with_config(modules, config);

    // 2. 分析 chunk 对之间的重叠
    let chunk_pair_overlaps = Self::analyze_chunk_pairs(chunks, &module_map, config);

    // 3. 计算总浪费空间
    let total_wasted_size = overlapped_modules.total_wasted_size();

    // 4. 生成优化建议
    let recommendations = Self::generate_recommendations(&overlapped_modules, &chunk_pair_overlaps);

    Self {
      overlapped_modules: overlapped_modules.into(),
      chunk_pair_overlaps,
      total_wasted_size,
      recommendations,
    }
  }

  /// 分析 chunk 对之间的重叠
  fn analyze_chunk_pairs(
    chunks: &[Chunk],
    module_map: &HashMap<String, &Module>,
    config: &ChunkOverlapConfig,
  ) -> Vec<ChunkPairOverlap> {
    let mut pairs = Vec::new();

    // 预先将所有 chunk 的 modules 转换为 HashSet（性能优化）
    let chunk_module_sets: Vec<HashSet<&String>> = chunks
      .iter()
      .map(|chunk| chunk.modules.iter().collect())
      .collect();

    // 两两比较 chunks
    for i in 0..chunks.len() {
      for j in (i + 1)..chunks.len() {
        let chunk_a = &chunks[i];
        let chunk_b = &chunks[j];

        // 使用 HashSet intersection 找出共享的模块（O(m) 而非 O(m²)）
        let shared_modules: Vec<String> = chunk_module_sets[i]
          .intersection(&chunk_module_sets[j])
          .map(|s| (*s).clone())
          .collect();

        if shared_modules.is_empty() {
          continue;
        }

        // 计算共享部分的大小
        let shared_size: u64 = shared_modules
          .iter()
          .filter_map(|mid| module_map.get(mid).map(|m| m.size))
          .sum();

        // 计算重叠比例
        let overlap_ratio_a = if chunk_a.size > 0 {
          shared_size as f64 / chunk_a.size as f64
        } else {
          0.0
        };

        let overlap_ratio_b = if chunk_b.size > 0 {
          shared_size as f64 / chunk_b.size as f64
        } else {
          0.0
        };

        // 检查重叠比例阈值
        if overlap_ratio_a < config.min_overlap_ratio && overlap_ratio_b < config.min_overlap_ratio
        {
          continue;
        }

        pairs.push(ChunkPairOverlap {
          chunk_a: chunk_a.id.clone(),
          chunk_b: chunk_b.id.clone(),
          shared_modules,
          shared_size,
          overlap_ratio_a,
          overlap_ratio_b,
        });
      }
    }

    // 按共享大小降序排序
    pairs.sort_by_key(|p| std::cmp::Reverse(p.shared_size));

    pairs
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
