use std::collections::{HashMap, HashSet};

use derive_more::derive::{Deref, Into};

use crate::{
  Chunk, Module,
  chunk_overlap::{ChunkOverlapConfig, ChunkPairOverlap},
};

#[derive(Debug, Deref, Into)]
pub struct ChunkPairOverlaps(pub Vec<ChunkPairOverlap>);

impl ChunkPairOverlaps {
  /// 分析 chunk 对之间的重叠
  pub fn from(chunks: &[Chunk], modules: &[Module]) -> Self {
    let config = ChunkOverlapConfig::default();
    Self::from_with_config(chunks, modules, &config)
  }

  /// 使用自定义配置分析 chunk 对之间的重叠
  pub fn from_with_config(chunks: &[Chunk], modules: &[Module], config: &ChunkOverlapConfig) -> Self {
    // 构建 module_id -> module 映射
    let module_map: HashMap<String, &Module> = modules.iter().map(|m| (m.id.clone(), m)).collect();

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

    Self(pairs)
  }
}
