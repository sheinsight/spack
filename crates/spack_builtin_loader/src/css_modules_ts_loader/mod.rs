use std::path::{Path, PathBuf};

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifier;
use rspack_core::{Loader, LoaderContext, RunnerContext};
use rspack_error::Result;
use serde::Serialize;
use strum_macros::EnumString;
use tokio::fs;

pub const CSS_MODULES_TS_LOADER_IDENTIFIER: &str = "builtin:css-modules-ts-loader";

#[cacheable]
#[derive(EnumString, Debug, Clone, Serialize)]
pub enum Mode {
  VERIFY,
  EMIT,
}

#[cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct CssModulesTsLoaderOpts {
  pub mode: Mode,
}

#[cacheable]
pub struct CssModulesTsLoader {
  options: CssModulesTsLoaderOpts,
}

impl CssModulesTsLoader {
  pub fn new(options: CssModulesTsLoaderOpts) -> Self {
    Self { options }
  }

  pub fn filename_to_typings_filename(&self, resource: &str) -> Result<PathBuf> {
    let p = Path::new(resource);
    let file_name = p
      .file_name()
      .ok_or_else(|| rspack_error::Error::error("invalid file name".to_string()))?;

    // let file_prefix = p
    //   .file_prefix()
    //   .ok_or_else(|| rspack_error::Error::error("invalid file prefix".to_string()))?;

    let dir = p
      .parent()
      .ok_or_else(|| rspack_error::Error::error("invalid file prefix".to_string()))?;

    let dts_file_name = dir.join(format!("{}.d.ts", file_name.to_string_lossy()));

    Ok(dts_file_name)
  }

  pub fn extract_local_exports(&self, content: &str) -> String {
    let mut local_exports = content.split("exports.locals").nth(1).unwrap_or("");
    if local_exports.is_empty() {
      local_exports = content
        .split("___CSS_LOADER_EXPORT___.locals")
        .nth(1)
        .unwrap_or("");
    }

    local_exports.to_string()
  }

  pub fn enforce_lf_line_separators(&self, text: Option<&str>) -> Option<String> {
    match text {
      Some(s) => Some(s.replace("\r\n", "\n")),
      None => None,
    }
  }
}

#[async_trait]
#[cacheable_dyn]
impl Loader<RunnerContext> for CssModulesTsLoader {
  fn identifier(&self) -> Identifier {
    CSS_MODULES_TS_LOADER_IDENTIFIER.into()
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let source = loader_context.take_content();

    let source_map = loader_context.take_source_map();

    let source_str = source
      .clone()
      .map(|v| v.into_string_lossy().to_string())
      .unwrap();

    let resource = loader_context.resource();

    let dts_file_name = self.filename_to_typings_filename(resource)?;

    let mut css_module_keys = rspack_util::fx_hash::FxHashSet::default();

    let key_regex = regex::Regex::new(r#""([^"\\]*(?:\\.[^"\\]*)*)""#).unwrap();

    let local_exports = self.extract_local_exports(&source_str);

    for cap in key_regex.captures_iter(&local_exports) {
      if let Some(matched_key) = cap.get(1) {
        css_module_keys.insert(matched_key.as_str().to_string());
      }
    }

    let dts_str = css_module_keys
      .iter()
      .map(|key| format!(r##"'{key}':string;"##))
      .collect::<Vec<String>>()
      .join("\n");

    let dts_str = format!(
      r#"
interface CssExports {{
  {dts_str}
}}
export const cssExports: CssExports;
export default cssExports;"#
    );

    if matches!(self.options.mode, Mode::VERIFY) {
      if dts_file_name.exists() {
        let existing_content = fs::read_to_string(&dts_file_name).await?;

        let left_str = self.enforce_lf_line_separators(Some(&existing_content));
        let right_str = self.enforce_lf_line_separators(Some(&dts_str));

        if left_str != right_str {
          return Err(rspack_error::Error::error(format!(
            "TypeScript definitions do not match for {:?}. Please regenerate.",
            dts_file_name
          )));
        }
      } else {
        return Err(rspack_error::Error::error(format!(
          "TypeScript definitions file {:?} does not exist. Please generate it.",
          dts_file_name
        )));
      }
    } else {
      fs::write(&dts_file_name, dts_str).await?;
    }

    loader_context.finish_with((source, source_map));
    Ok(())
  }
}
