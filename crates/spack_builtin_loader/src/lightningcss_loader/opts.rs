use lightningcss::targets::Browsers;
use rspack_cacheable::{
  cacheable,
  with::{AsOption, AsPreset},
};

use crate::lightningcss_loader::visitors::px_to_rem::PxToRemOpts;

#[cacheable]
#[derive(Debug, Clone)]
pub struct Draft {
  pub custom_media: bool,
  pub px_to_rem: Option<PxToRemOpts>,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct NonStandard {
  pub deep_selector_combinator: bool,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct PseudoClasses {
  pub hover: Option<String>,
  pub active: Option<String>,
  pub focus: Option<String>,
  pub focus_visible: Option<String>,
  pub focus_within: Option<String>,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct LightningcssLoaderOpts {
  pub minify: bool,
  #[cacheable(with=AsOption<AsPreset>)]
  pub targets: Option<Browsers>,
  pub error_recovery: bool,
  // pub include: Option<u32>,
  // pub exclude: Option<u32>,
  pub draft: Option<Draft>,
  pub non_standard: Option<NonStandard>,
  pub pseudo_classes: Option<PseudoClasses>,
  pub unused_symbols: Option<Vec<String>>,
}
