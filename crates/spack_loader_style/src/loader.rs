use std::collections::HashMap;

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifier;
use rspack_core::{Loader, LoaderContext, RunnerContext};
use rspack_error::Result;
use rspack_loader_runner::Identifiable;
use serde::Serialize;
use strum_macros::{Display, EnumString};

// use crate::template;

#[cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct StyleLoaderOpts {
  pub base: Option<i64>,
  pub inject_type: Option<InjectType>,
  pub es_module: Option<bool>,
  pub insert: Option<String>,
  pub output: String,
  pub attributes: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Copy, Serialize, Display, EnumString)]
#[cacheable]
#[strum(serialize_all = "camelCase")]
pub enum InjectType {
  StyleTag,
  SingletonStyleTag,
  AutoStyleTag,
  LazyStyleTag,
  LazySingletonStyleTag,
  LazyAutoStyleTag,
  LinkTag,
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
    // let source = "".to_string();

    let source = loader_context.take_content();
    let sm = loader_context.take_source_map();
    let request = loader_context.resource_query();

    let ctx = crate::temp::LinkHmrCodeTemplate {
      name: request.unwrap_or_default().to_string(),
    };

    println!("{}", sailfish::TemplateSimple::render_once(&ctx).unwrap());

    println!(
      r##"
  ======================= pitch ========================

  request:
  
  {:#?}
  
  
  
  source:
  
  {:#?}
  
  
  sm:
  
  {:#?}
  ======================= pitch ========================
  "##,
      request,
      source.clone().map(|s| s.try_into_string()),
      sm.clone()
    );
    loader_context.finish_with((source, sm));
    Ok(())
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    // let source = "".to_string();
    let source = loader_context.take_content();
    let sm = loader_context.take_source_map();
    let request = loader_context.request();
    println!(
      r##"
======================= run ========================

request: 

{:#?}


source:

{:#?}


sm:

{:#?}
======================= run ========================
"##,
      request,
      source.clone().map(|s| s.try_into_string()),
      sm.clone(),
    );
    loader_context.finish_with((source, sm));
    Ok(())
  }
}

// impl Identifiable for StyleLoader {
//   fn identifier(&self) -> Identifier {
//     STYLE_LOADER_IDENTIFIER.into()
//   }
// }
