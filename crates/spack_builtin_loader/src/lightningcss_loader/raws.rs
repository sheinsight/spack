use rspack_browserslist::browserslist_to_lightningcss_targets;
use rspack_error::ToStringResultToRspackResultExt;
use serde::{Deserialize, Serialize};

use crate::lightningcss_loader::{
  opts::{Draft, LightningcssLoaderOpts, NonStandard, PseudoClasses},
  visitors::px_to_rem::RawPxToRemOpts,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawLightningcssLoaderOpts {
  pub minify: Option<bool>,
  pub targets: Option<Vec<String>>,
  pub error_recovery: Option<bool>,
  pub draft: Option<RawDraft>,
  pub non_standard: Option<RawNonStandard>,
  pub pseudo_classes: Option<RawPseudoClasses>,
  pub unused_symbols: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawDraft {
  pub custom_media: Option<bool>,
  pub px_to_rem: Option<RawPxToRemOpts>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawNonStandard {
  pub deep_selector_combinator: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawPseudoClasses {
  pub hover: Option<String>,
  pub active: Option<String>,
  pub focus: Option<String>,
  pub focus_visible: Option<String>,
  pub focus_within: Option<String>,
}

impl TryInto<LightningcssLoaderOpts> for RawLightningcssLoaderOpts {
  type Error = rspack_error::Error;
  fn try_into(self) -> Result<LightningcssLoaderOpts, Self::Error> {
    let targets = self
      .targets
      .map(browserslist_to_lightningcss_targets)
      .transpose()
      .to_rspack_result_with_message(|e| format!("Failed to parse browserslist: {e}"))?
      .flatten();

    Ok(LightningcssLoaderOpts {
      minify: self.minify.unwrap_or(true),
      targets,
      error_recovery: self.error_recovery.unwrap_or(false),
      draft: self.draft.map(Into::into),
      non_standard: self.non_standard.map(Into::into),
      pseudo_classes: self.pseudo_classes.map(Into::into),
      unused_symbols: self.unused_symbols,
    })
  }
}

impl Into<Draft> for RawDraft {
  fn into(self) -> Draft {
    Draft {
      custom_media: self.custom_media.unwrap_or(false),
      px_to_rem: self.px_to_rem.map(Into::into),
    }
  }
}

impl Into<NonStandard> for RawNonStandard {
  fn into(self) -> NonStandard {
    NonStandard {
      deep_selector_combinator: self.deep_selector_combinator.unwrap_or(false),
    }
  }
}

impl Into<PseudoClasses> for RawPseudoClasses {
  fn into(self) -> PseudoClasses {
    PseudoClasses {
      hover: self.hover,
      active: self.active,
      focus: self.focus,
      focus_visible: self.focus_visible,
      focus_within: self.focus_within,
    }
  }
}
