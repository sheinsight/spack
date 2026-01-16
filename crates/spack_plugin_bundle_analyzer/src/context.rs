use std::collections::HashMap;

use rspack_collections::Identifier;
use rspack_core::Compilation;

/// Module 和 Chunk 之间的映射关系上下文
///
/// 通过一次遍历 chunk_graph 构建，避免在 Modules 和 Chunks 收集时重复遍历
pub struct ModuleChunkContext {
  /// module_id -> chunk_ids 映射
  pub module_to_chunks: HashMap<Identifier, Vec<String>>,
  /// chunk_id -> (module_ids, total_size) 映射
  pub chunk_to_modules: HashMap<String, (Vec<String>, u64)>,
}

impl<'a> From<&'a Compilation> for ModuleChunkContext {
  fn from(compilation: &'a Compilation) -> Self {
    let chunk_graph = &compilation.chunk_graph;
    let module_graph = compilation.get_module_graph();

    let mut module_to_chunks: HashMap<Identifier, Vec<String>> = HashMap::new();
    let mut chunk_to_modules: HashMap<String, (Vec<String>, u64)> = HashMap::new();

    // 只遍历一次 chunk_graph，同时构建双向映射
    for (ukey, _chunk) in compilation.chunk_by_ukey.iter() {
      let chunk_id = ukey.as_u32().to_string();

      let (module_ids, size) = chunk_graph
        .get_chunk_modules(ukey, &module_graph)
        .iter()
        .fold((Vec::new(), 0u64), |(mut ids, total_size), m| {
          let module_id = m.identifier();
          ids.push(module_id.to_string());

          // 构建 module → chunks 反向映射
          module_to_chunks
            .entry(module_id)
            .or_insert_with(Vec::new)
            .push(chunk_id.clone());

          let module_size = m.size(None, None) as u64;
          (ids, total_size + module_size)
        });

      chunk_to_modules.insert(chunk_id, (module_ids, size));
    }

    Self {
      module_to_chunks,
      chunk_to_modules,
    }
  }
}
