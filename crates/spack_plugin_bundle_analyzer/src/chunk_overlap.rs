use std::collections::HashMap;

use derive_more::derive::{Deref, Into};

use crate::{Chunk, Module, package_version_resolver::PackageVersionResolver};

/// 重叠模块信息
#[derive(Debug, Clone)]
pub struct OverlappedModule {
  /// 模块 ID
  pub module_id: String,
  /// 模块可读名称
  pub module_name: String,
  /// 模块大小（字节）
  pub module_size: u64,
  /// 包含此模块的 chunk IDs
  pub chunks: Vec<String>,
  /// 重复次数（chunks.len()）
  pub duplication_count: usize,
  /// 浪费的空间（size * (count - 1)）
  pub wasted_size: u64,
  /// 包名（如果是 node_modules 中的模块）
  pub package_name: Option<String>,
}

/// Chunk 对之间的重叠信息
#[derive(Debug, Clone)]
pub struct ChunkPairOverlap {
  /// Chunk A 的 ID
  pub chunk_a: String,
  /// Chunk B 的 ID
  pub chunk_b: String,
  /// 共享的模块 ID 列表
  pub shared_modules: Vec<String>,
  /// 共享部分的总大小（字节）
  pub shared_size: u64,
  /// 占 Chunk A 的比例
  pub overlap_ratio_a: f64,
  /// 占 Chunk B 的比例
  pub overlap_ratio_b: f64,
}

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

/// Chunk 重叠分析配置
#[derive(Debug, Clone)]
pub struct ChunkOverlapConfig {
  /// 最小模块大小阈值（小于此不报告）
  pub min_module_size: u64,
  /// 最小重复次数（少于此不报告）
  pub min_duplication_count: usize,
  /// 最小浪费空间阈值（小于此不报告）
  pub min_wasted_size: u64,
  /// Chunk 对重叠比例阈值（低于此不报告）
  pub min_overlap_ratio: f64,
  /// 是否包含内部模块（非 node_modules）
  pub include_internal_modules: bool,
}

impl Default for ChunkOverlapConfig {
  fn default() -> Self {
    Self {
      // 1KB - 太小的模块不值得优化
      min_module_size: 1024,
      // 至少重复 2 次
      min_duplication_count: 2,
      // 浪费至少 10KB 才报告
      min_wasted_size: 10 * 1024,
      // chunk 对重叠至少 10%
      min_overlap_ratio: 0.1,
      // 包含内部模块
      include_internal_modules: true,
    }
  }
}

#[derive(Debug, Deref, Into)]
pub struct ChunkOverlapAnalyses(pub ChunkOverlapAnalysis);

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
    let module_map: HashMap<String, &Module> = modules
      .iter()
      .map(|m| (m.id.clone(), m))
      .collect();

    // 1. 找出重叠的模块
    let overlapped_modules = Self::find_overlapped_modules(modules, config);

    // 2. 分析 chunk 对之间的重叠
    let chunk_pair_overlaps =
      Self::analyze_chunk_pairs(chunks, &module_map, config);

    // 3. 计算总浪费空间
    let total_wasted_size: u64 = overlapped_modules
      .iter()
      .map(|m| m.wasted_size)
      .sum();

    // 4. 生成优化建议
    let recommendations =
      Self::generate_recommendations(&overlapped_modules, &chunk_pair_overlaps);

    Self {
      overlapped_modules,
      chunk_pair_overlaps,
      total_wasted_size,
      recommendations,
    }
  }

  /// 找出重叠的模块
  fn find_overlapped_modules(
    modules: &[Module],
    config: &ChunkOverlapConfig,
  ) -> Vec<OverlappedModule> {
    let mut overlapped = Vec::new();
    let mut resolver = PackageVersionResolver::new();

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
      let package_name = if module.is_node_module {
        resolver.resolve(&module.name).map(|info| info.name)
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

    overlapped
  }

  /// 分析 chunk 对之间的重叠
  fn analyze_chunk_pairs(
    chunks: &[Chunk],
    module_map: &HashMap<String, &Module>,
    config: &ChunkOverlapConfig,
  ) -> Vec<ChunkPairOverlap> {
    let mut pairs = Vec::new();

    // 两两比较 chunks
    for i in 0..chunks.len() {
      for j in (i + 1)..chunks.len() {
        let chunk_a = &chunks[i];
        let chunk_b = &chunks[j];

        // 找出共享的模块
        let shared_modules: Vec<String> = chunk_a
          .modules
          .iter()
          .filter(|m| chunk_b.modules.contains(m))
          .cloned()
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
        if overlap_ratio_a < config.min_overlap_ratio
          && overlap_ratio_b < config.min_overlap_ratio
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
