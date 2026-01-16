use derive_more::derive::{Deref, Into};
use rspack_core::Compilation;

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
    let chunk_graph = &compilation.chunk_graph;
    let module_graph = compilation.get_module_graph();

    let chunks = compilation
      .chunk_by_ukey
      .iter()
      .map(|(ukey, chunk)| {
        // 一次遍历同时收集模块ID和计算大小，避免二次查找
        let (modules, size) = chunk_graph
          .get_chunk_modules(ukey, &module_graph)
          .iter()
          .fold((Vec::new(), 0u64), |(mut ids, total_size), m| {
            ids.push(m.identifier().to_string());
            let module_size = get_module_size(m.as_ref());
            (ids, total_size + module_size)
          });

        let id = ukey.as_u32().to_string();
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

fn get_module_size(module: &dyn rspack_core::Module) -> u64 {
  // 使用 Module trait 的 size 方法获取模块大小
  // source_type 参数为 None 表示获取所有类型的总大小
  // compilation 参数为 None 因为我们不需要编译上下文
  module.size(None, None) as u64
}
