use rspack_cacheable::cacheable;
use serde::{Deserialize, Serialize};

use crate::lightningcss_loader::px_to_rem::PxToRemOpts;

#[cacheable]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
  pub custom_media: bool,
  pub px_to_rem: Option<PxToRemOpts>,
}

#[cacheable]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NonStandard {
  pub deep_selector_combinator: bool,
}

#[cacheable]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PseudoClasses {
  pub hover: Option<String>,
  pub active: Option<String>,
  pub focus: Option<String>,
  pub focus_visible: Option<String>,
  pub focus_within: Option<String>,
}

#[cacheable]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningcssLoaderOpts {
  /// default is 16
  // root_value: Option<i32>,

  /// default is 5
  // unit_precision: Option<i32>,
  pub minify: Option<bool>,
  pub error_recovery: Option<bool>,
  // #[cacheable(with=AsOption<AsPreset>)]
  // pub targets: Option<Browsers>,
  pub include: Option<u32>,
  pub exclude: Option<u32>,
  pub draft: Option<Draft>,
  pub non_standard: Option<NonStandard>,
  pub pseudo_classes: Option<PseudoClasses>,
  pub unused_symbols: Option<Vec<String>>,
}
