use async_trait::async_trait;
use lightningcss::rules::CssRule;
use lightningcss::stylesheet::{ParserOptions, StyleSheet};
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifier;
use rspack_core::{Loader, LoaderContext, RunnerContext};
use rspack_error::Result;
use serde::Serialize;

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

    let style_sheet = StyleSheet::parse(source, parser_options)
      .map_err(|_e| rspack_error::Error::error("parse css error".to_string()))?;
    // println!("style_sheet--->{:?}", style_sheet);

    // 遍历所有规则
    for rule in style_sheet.rules.0.iter() {
      match rule {
        CssRule::Import(import_rule) => {
          // 提取 import 的 URL
          // let url = import_rule.url.to_string();
          let url = import_rule.url.clone();
          println!("Found @import: {:?}", url);

          // 打印媒体查询信息
          // println!("  Media query: {:?}", import_rule.media);
        }
        CssRule::Style(style_rule) => {
          // 处理样式规则，查找 background-image 等属性
          println!("Found style rule: {:?}", style_rule);
          for (declaration, _) in style_rule.declarations.iter() {
            println!("Found declaration: {:?}", declaration);
          }
        }
        _ => {
          // 其他类型的规则，可以在这里处理
        }
      }
    }

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

    self.parse_css(&raw.try_into_string()?, loader_context.resource())?;

    loader_context.finish_with((source, source_map));
    Ok(())
  }
}
