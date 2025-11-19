use std::path::{Path, PathBuf};

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifier;
use rspack_core::{Loader, LoaderContext, RunnerContext};
use rspack_error::Result;
use rspack_util::fx_hash::FxHashSet;
use serde::Serialize;
use strum_macros::EnumString;
use tokio::fs;

pub const CSS_MODULES_TS_LOADER_IDENTIFIER: &str = "builtin:css-modules-dts-loader";

#[cacheable]
#[derive(EnumString, Debug, Clone, Serialize)]
pub enum Mode {
  VERIFY,
  EMIT,
}

#[cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct CssModulesDtsLoaderOpts {
  pub mode: Mode,
}

#[cacheable]
pub struct CssModulesDtsLoader {
  options: CssModulesDtsLoaderOpts,
}

impl CssModulesDtsLoader {
  pub fn new(options: CssModulesDtsLoaderOpts) -> Self {
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

  fn extract_banner_keys(&self, content: &str) -> FxHashSet<String> {
    let banner_regex = regex::Regex::new(r"// Banner\s*:\s*(.*)").unwrap();
    banner_regex
      .captures(content)
      .and_then(|cap| cap.get(1).map(|m| m.as_str().trim()))
      .map(|banner| banner.split(',').map(|s| s.trim().to_string()).collect())
      .unwrap_or_default()
  }
}

#[async_trait]
#[cacheable_dyn]
impl Loader<RunnerContext> for CssModulesDtsLoader {
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

    let mut css_module_keys = FxHashSet::default();

    let key_regex = regex::Regex::new(r#""([^"\\]*(?:\\.[^"\\]*)*)""#)
      .map_err(|e| rspack_error::Error::error(format!("Failed to compile regex: {}", e)))?;

    let local_exports = self.extract_local_exports(&source_str);

    for cap in key_regex.captures_iter(&local_exports) {
      if let Some(matched_key) = cap.get(1) {
        css_module_keys.insert(matched_key.as_str().to_string());
      }
    }

    let keys_vec: Vec<String> = css_module_keys.iter().cloned().collect();

    let banner_str = keys_vec.join(",");

    let dts_str = css_module_keys
      .iter()
      .map(|key| format!(r##"'{key}':string;"##))
      .collect::<Vec<_>>()
      .join("\n");

    let dts_content = format!(
      r#"// Please do not delete the comments, as they are used to determine content changes.
// Banner: {banner_str}
interface CssExports {{
  {dts_str}
}}
export const cssExports: CssExports;
export default cssExports;"#
    );

    if !matches!(self.options.mode, Mode::VERIFY) {
      fs::write(&dts_file_name, dts_content).await?;
      loader_context.finish_with((source, source_map));
      return Ok(());
    }

    if !dts_file_name.exists() {
      return Err(rspack_error::Error::error(format!(
        "TypeScript definitions file {:?} does not exist. Please generate it.",
        dts_file_name
      )));
    }

    let existing_content = fs::read_to_string(&dts_file_name).await?;

    let existing_keys = self.extract_banner_keys(&existing_content);

    let diff = css_module_keys.difference(&existing_keys);

    if diff.count() > 0 {
      return Err(rspack_error::Error::error(format!(
        "TypeScript definitions do not match for {:?}. Please regenerate.",
        dts_file_name
      )));
    }

    loader_context.finish_with((source, source_map));

    Ok(())
  }
}
