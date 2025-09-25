use std::{collections::HashMap, path::PathBuf};

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifier;
use rspack_core::{Loader, LoaderContext, RunnerContext, contextify};
use rspack_error::Result;
use rspack_loader_runner::DisplayWithSuffix;
use serde::Serialize;

use crate::ModuleHelper;

#[cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct CssLoaderOpts {}

#[cacheable]
pub struct CssLoader {
  options: CssLoaderOpts,
}

impl CssLoader {
  pub fn new(options: CssLoaderOpts) -> Self {
    Self { options }
  }
}

pub const CSS_LOADER_IDENTIFIER: &str = "builtin:css-loader";

#[async_trait]
#[cacheable_dyn]
impl Loader<RunnerContext> for CssLoader {
  fn identifier(&self) -> Identifier {
    CSS_LOADER_IDENTIFIER.into()
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let source = loader_context.take_content();
    let sm = loader_context.take_source_map();

    println!("source--->{:?}", source);

    loader_context.finish_with((source, sm));
    Ok(())
  }
}
