use std::{collections::HashMap, path::PathBuf};

use async_trait::async_trait;
use lightningcss::stylesheet::{ParserOptions, StyleSheet};
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

  /// 使用 lightningcss 解析 CSS 源码
  fn parse_css(&self, source: &str, filename: &str) -> Result<()> {
    let parser_options = ParserOptions {
      filename: filename.to_string(),
      ..Default::default()
    };

    let stylesheet = StyleSheet::parse(source, parser_options)
      .map_err(|e| rspack_error::Error::error("parse css error".to_string()))?;
    println!("stylesheet--->{:?}", stylesheet);
    // Ok(stylesheet)
    Ok(())
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
    let source_map = loader_context.take_source_map();

    let Some(raw) = source.clone() else {
      return Ok(());
    };

    println!("source--->{:?}", raw.clone());

    let stylesheet = self.parse_css(&raw.try_into_string()?, loader_context.resource())?;
    println!("stylesheet--->{:?}", stylesheet);

    loader_context.finish_with((source, source_map));
    Ok(())
  }
}
