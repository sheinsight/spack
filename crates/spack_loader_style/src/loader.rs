use std::collections::HashMap;

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifier;
use rspack_core::{Loader, LoaderContext, RunnerContext};
use rspack_error::Result;
use rspack_loader_runner::DisplayWithSuffix;
use serde::Serialize;

use crate::InjectType;

#[cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct StyleLoaderOpts {
  pub base: Option<i64>,
  pub inject_type: Option<InjectType>,
  pub insert: Option<String>,
  pub home_dir: String,
  pub output_dir: String,
  pub style_tag_transform: Option<String>,
  pub attributes: Option<HashMap<String, String>>,
}

#[cacheable]
pub struct StyleLoader {
  pub options: StyleLoaderOpts,
}

pub const STYLE_LOADER_IDENTIFIER: &str = "builtin:style-loader";

#[async_trait]
#[cacheable_dyn]
impl Loader<RunnerContext> for StyleLoader {
  fn identifier(&self) -> Identifier {
    STYLE_LOADER_IDENTIFIER.into()
  }
  async fn pitch(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let source_map = loader_context.take_source_map();

    let resource = loader_context.resource();

    let request = loader_context.remaining_request();

    let request = request.display_with_suffix(resource);

    let inject_type = self.options.inject_type.unwrap_or_default();

    let source = inject_type.code(&request, loader_context, &self.options);

    loader_context.finish_with((source, source_map));
    Ok(())
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let source = loader_context.take_content();
    let sm = loader_context.take_source_map();

    loader_context.finish_with((source, sm));
    Ok(())
  }
}
