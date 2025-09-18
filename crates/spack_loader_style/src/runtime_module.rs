use rspack_collections::Identifier;
use rspack_core::{impl_runtime_module, ChunkUkey, Compilation, RuntimeModule};

#[impl_runtime_module]
#[derive(Debug)]
pub struct StyleLoaderRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  es_module: bool,
}

impl StyleLoaderRuntimeModule {
  pub fn new(chunk: Option<ChunkUkey>, es_module: bool) -> Self {
    Self::with_default(
      Identifier::from(format!("webpack/runtime/link_tag")),
      chunk,
      es_module,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for StyleLoaderRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![
      // (
      //   self.id.to_string(),
      //   include_str!("runtime/link_tag.ejs").to_string(),
      // ),
      (
        self.id.to_string(),
        include_str!("runtime/link_hmr_code.ejs").to_string(),
      ),
    ]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let source = compilation.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "esModule":  self.es_module
      })),
    )?;

    Ok(source)
  }
}
