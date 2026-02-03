use std::collections::HashMap;

use derive_more::derive::{Deref, Into};
use rspack_core::Compilation;

use crate::{Chunk, ModuleIdMapper, context::ModuleChunkContext};

#[derive(Debug, Default, Deref, Into)]
pub struct Chunks(Vec<Chunk>);

impl<'a> From<&'a mut Compilation> for Chunks {
  fn from(compilation: &'a mut Compilation) -> Self {
    // 保留原有实现用于向后兼容
    let context = ModuleChunkContext::from(&*compilation);
    Self::from_with_context(compilation, &context)
  }
}

impl Chunks {
  /// 使用预构建的上下文来避免重复遍历 chunk_graph
  ///
  /// 参数:
  /// - compilation: 编译上下文
  /// - context: 预构建的 module ↔ chunk 映射关系
  pub fn from_with_context(compilation: &mut Compilation, context: &ModuleChunkContext) -> Self {
    // 保留原有实现用于向后兼容（不使用 ID 映射）
    let id_mapper = ModuleIdMapper::new();
    Self::from_with_context_and_mapper(compilation, context, &id_mapper)
  }

  /// 使用预构建的上下文和 ID 映射器来避免重复遍历 chunk_graph
  ///
  /// 参数:
  /// - compilation: 编译上下文
  /// - context: 预构建的 module ↔ chunk 映射关系
  /// - id_mapper: 模块 ID 映射器（用于将原始路径转换为数字 ID）
  pub fn from_with_context_and_mapper(
    compilation: &mut Compilation,
    context: &ModuleChunkContext,
    id_mapper: &ModuleIdMapper,
  ) -> Self {
    let chunk_graph = &compilation.chunk_graph;

    // 预先构建 chunk 的 parents 和 children 映射
    let (parents_map, children_map) = build_chunk_relations(compilation);

    let chunks = compilation
      .chunk_by_ukey
      .iter()
      .map(|(ukey, chunk)| {
        let id = ukey.as_u32().to_string();

        // O(1) 查找，不需要再次遍历 chunk_graph
        // 将模块 ID 转换为数字 ID
        let (module_ids, size) = context
          .chunk_to_modules
          .get(&id)
          .cloned()
          .unwrap_or_default();

        // 转换模块 ID 为数字 ID
        let modules: Vec<u32> = module_ids
          .iter()
          .filter_map(|module_id| id_mapper.get(module_id))
          .collect();

        let names = chunk
          .name()
          .map(|n| vec![n.to_string()])
          .unwrap_or_default();
        let files = chunk.files().iter().cloned().collect();
        let reason = chunk.chunk_reason().unwrap_or_default().to_string();
        let initial = chunk.can_be_initial(&compilation.chunk_group_by_ukey);
        let entry = chunk.has_entry_module(chunk_graph);
        let async_chunks = chunk.has_async_chunks(&compilation.chunk_group_by_ukey);
        let runtime = chunk.has_runtime(&compilation.chunk_group_by_ukey);

        let parents = parents_map.get(&id).cloned().unwrap_or_default();
        let children = children_map.get(&id).cloned().unwrap_or_default();

        Chunk {
          id,
          names,
          size,
          modules, // 使用数字 ID 列表
          entry,
          initial,
          reason,
          files,
          async_chunks,
          runtime,
          parents,
          children,
        }
      })
      .collect();

    Chunks(chunks)
  }
}

/// 构建 chunk 的 parents 和 children 关系映射
///
/// 返回: (parents_map, children_map)
/// - parents_map: chunk_id -> parent_chunk_ids
/// - children_map: chunk_id -> child_chunk_ids
fn build_chunk_relations(
  compilation: &Compilation,
) -> (HashMap<String, Vec<String>>, HashMap<String, Vec<String>>) {
  let mut parents_map: HashMap<String, Vec<String>> = HashMap::new();
  let mut children_map: HashMap<String, Vec<String>> = HashMap::new();

  let chunk_group_by_ukey = &compilation.chunk_group_by_ukey;

  // 遍历所有 chunk groups
  for (_group_ukey, group) in chunk_group_by_ukey.iter() {
    // 获取当前 group 的所有 chunks
    let current_chunks: Vec<String> = group
      .chunks
      .iter()
      .map(|chunk_ukey| chunk_ukey.as_u32().to_string())
      .collect();

    // 获取父 groups 的所有 chunks
    let parent_chunks: Vec<String> = group
      .parents
      .iter()
      .flat_map(|parent_group_ukey| {
        chunk_group_by_ukey
          .get(parent_group_ukey)
          .map(|parent_group| {
            parent_group
              .chunks
              .iter()
              .map(|chunk_ukey| chunk_ukey.as_u32().to_string())
              .collect::<Vec<_>>()
          })
          .unwrap_or_default()
      })
      .collect();

    // 获取子 groups 的所有 chunks
    let child_chunks: Vec<String> = group
      .children
      .iter()
      .flat_map(|child_group_ukey| {
        chunk_group_by_ukey
          .get(child_group_ukey)
          .map(|child_group| {
            child_group
              .chunks
              .iter()
              .map(|chunk_ukey| chunk_ukey.as_u32().to_string())
              .collect::<Vec<_>>()
          })
          .unwrap_or_default()
      })
      .collect();

    // 为当前 group 的每个 chunk 设置 parents 和 children
    for chunk_id in &current_chunks {
      // 添加 parents
      if !parent_chunks.is_empty() {
        parents_map
          .entry(chunk_id.clone())
          .or_default()
          .extend(parent_chunks.clone());
      }

      // 添加 children
      if !child_chunks.is_empty() {
        children_map
          .entry(chunk_id.clone())
          .or_default()
          .extend(child_chunks.clone());
      }
    }
  }

  // 去重并排序
  for (_, parents) in parents_map.iter_mut() {
    parents.sort();
    parents.dedup();
  }

  for (_, children) in children_map.iter_mut() {
    children.sort();
    children.dedup();
  }

  (parents_map, children_map)
}
