use rspack_cacheable::cacheable;
use rspack_collections::Identifier;
use rspack_core::{ChunkUkey, Compilation, RuntimeModule, impl_runtime_module};
use strum_macros::{Display, EnumString};

#[impl_runtime_module]
#[derive(Debug)]
pub struct StyleLoaderRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  es_module: bool,
}

// #[cacheable]
// #[derive(Debug, Clone, EnumString, Display)]
// pub enum StyleLoaderTemplateType {
//   LinkTag,
//   LinkHmrCode,
// }

impl StyleLoaderRuntimeModule {
  pub fn new(chunk: Option<ChunkUkey>, es_module: bool) -> Self {
    Self::with_default(
      Identifier::from(format!("style_loader/runtime")),
      chunk,
      es_module,
    )
  }

  // fn get_template_id(&self, mode: &StyleLoaderTemplateType) -> String {
  //   match mode {
  //     StyleLoaderTemplateType::LinkTag => format!("{}_{}", &self.id, mode.to_string()),
  //     StyleLoaderTemplateType::LinkHmrCode => format!("{}_{}", &self.id, mode.to_string()),
  //   }
  // }
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
      //   "link_tag".to_string(),
      //   include_str!("runtime/link_tag.ejs").to_string(),
      // ),
      // (
      //   "link_hmr_code".to_string(),
      //   include_str!("runtime/link_hmr_code.ejs").to_string(),
      // ),
    ]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let template_params = serde_json::json!({
      "esModule":  self.es_module
    });

    let mut sources = Vec::new();

    for (template_id, _) in self.template() {
      let source = compilation
        .runtime_template
        .render(&template_id, Some(template_params.clone()))?;

      sources.push(source);
    }

    Ok(sources.join("\n"))
  }
}
