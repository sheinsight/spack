use async_trait::async_trait;
use lightningcss::css_modules::{Config, Pattern};
use lightningcss::properties::Property;
use lightningcss::rules::CssRule;
use lightningcss::stylesheet::{ParserOptions, StyleSheet};
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifier;
use rspack_core::{Loader, LoaderContext, RunnerContext};
use rspack_error::Result;
use serde::Serialize;

pub enum Modules {
  False,
  Configured(Configured),
}

pub struct Configured {
  local_ident_name: String,
}

#[cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct CssLoaderOpts {
  pub modules: String,
  pub import_loaders: i32,
}

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
      css_modules: Some(Config {
        pattern: Pattern::default(),
        dashed_idents: false,
        animation: true,
        grid: true,
        container: true,
        custom_idents: true,
        pure: false,
      }),
      ..Default::default()
    };

    let style_sheet = StyleSheet::parse(source, parser_options)
      .map_err(|_e| rspack_error::Error::error("parse css error".to_string()))?;
    // println!("style_sheet--->{:?}", style_sheet);

    let mut sources = Vec::new();

    // 遍历所有规则
    for (index, rule) in style_sheet.rules.0.iter().enumerate() {
      match rule {
        CssRule::Import(import_rule) => {
          // 提取 import 的 URL
          // let url = import_rule.url.to_string();
          let url = import_rule.url.clone();
          println!("Found @import: {:?}", import_rule);

          sources.push(format!(
            "import ___CSS_LOADER_AT_RULE_IMPORT_{index}___ from '{url}';"
          ));

          // 打印媒体查询信息
          // println!("  Media query: {:?}", import_rule.media);
        }
        CssRule::Style(style_rule) => {
          // 处理样式规则，查找 background-image 等属性
          // println!("Found style rule: {:?}", style_rule);
          for (declaration, _) in style_rule.declarations.iter() {
            // println!("Found declaration: {:?}", declaration);
            match declaration {
              Property::BackgroundImage(background_image) => {
                for image in background_image.iter() {
                  match image {
                    lightningcss::values::image::Image::None => todo!(),
                    lightningcss::values::image::Image::Url(url) => {
                      println!("Found background-image: {:?}", url);
                      let url_str = url.url.to_string();
                      let code = format!(
                        r##"
          var ___CSS_LOADER_URL_IMPORT_0___ = new URL("{url_str}", import.meta.url);
          var ___CSS_LOADER_URL_REPLACEMENT_0___ = ___CSS_LOADER_GET_URL_IMPORT___(___CSS_LOADER_URL_IMPORT_0___);
          .bg {{ background: url(___CSS_LOADER_URL_REPLACEMENT_0___); }}
  "##
                      );
                      sources.push(code);
                    }
                    lightningcss::values::image::Image::Gradient(_gradient) => todo!(),
                    lightningcss::values::image::Image::ImageSet(_image_set) => todo!(),
                  }
                }
              }
              _ => {}
            }
          }
        }
        _ => {
          // 其他类型的规则，可以在这里处理
        }
      }
    }

    println!(
      r##"
---sources---
{}
---sources---
"##,
      sources.join("\n")
    );

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
