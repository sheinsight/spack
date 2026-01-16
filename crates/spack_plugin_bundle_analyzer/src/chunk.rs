use derive_more::derive::{Deref, Into};
use rspack_core::Compilation;

use crate::context::ModuleChunkContext;

#[derive(Debug)]
pub struct Chunk {
  // chunk ID
  pub id: String,
  // chunk 名称
  pub names: Vec<String>,
  // chunk 大小
  pub size: u64,
  // 包含的模块 ID 列表
  pub modules: Vec<String>,
  // 是否入口 chunk
  pub entry: bool,
  // 是否初始 chunk
  pub initial: bool,
  // chunk 创建的原因(如 entry、import()、splitChunks 等)
  pub reason: String,
  // chunk 生成的输出文件列表
  pub files: Vec<String>,
  // 是否包含异步 chunk
  pub async_chunks: bool,
  // 是否包含运行时代码
  pub runtime: bool,
}

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
  pub fn from_with_context(
    compilation: &mut Compilation,
    context: &ModuleChunkContext,
  ) -> Self {
    let chunk_graph = &compilation.chunk_graph;

    let chunks = compilation
      .chunk_by_ukey
      .iter()
      .map(|(ukey, chunk)| {
        let id = ukey.as_u32().to_string();

        // O(1) 查找，不需要再次遍历 chunk_graph
        let (modules, size) = context
          .chunk_to_modules
          .get(&id)
          .cloned()
          .unwrap_or_default();

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

        Chunk {
          id,
          names,
          size,
          modules,
          entry,
          initial,
          reason,
          files,
          async_chunks,
          runtime,
        }
      })
      .collect();

    Chunks(chunks)
  }
}
