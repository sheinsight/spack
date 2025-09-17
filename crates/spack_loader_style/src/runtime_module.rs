use rspack_collections::Identifier;
use rspack_core::{impl_runtime_module, Compilation, RuntimeModule};

#[impl_runtime_module]
#[derive(Debug)]
pub struct StyleLoaderRuntimeModule {
  id: Identifier,
}

impl StyleLoaderRuntimeModule {
  pub fn new() -> Self {
    Self::with_default(Identifier::from(format!("webpack/runtime/link_tag")))
  }
}

#[async_trait::async_trait]
impl RuntimeModule for StyleLoaderRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/link_tag.ejs").to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let source = compilation.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "RUNTIME_HANDLERS":  "&self.runtime_handlers.to_string()".to_string(),
        "RUNTIME_FUNCTION": "&self.runtime_function.to_string()".to_string(),
      })),
    )?;

    Ok(source)
  }
}
