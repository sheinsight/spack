use lightningcss::targets::Browsers;
use rspack_cacheable::{
  cacheable,
  with::{AsOption, AsPreset},
};

use crate::lightningcss_loader::px_to_rem::PxToRemOpts;

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
  // #[cacheable(with=AsOption<AsPreset>)]
  // pub targets: Option<Browsers>,
  // pub include: Option<u32>,
  // pub exclude: Option<u32>,
  pub draft: Option<Draft>,
  pub non_standard: Option<NonStandard>,
  pub pseudo_classes: Option<PseudoClasses>,
  pub unused_symbols: Option<Vec<String>>,
}

// impl TryFrom<RawLightningcssLoaderOpts> for LightningcssLoaderOpts {
//   type Error = rspack_error::Error;
//   fn try_from(value: RawLightningcssLoaderOpts) -> Result<Self, Self::Error> {
//     Ok(Self {
//       minify: value.minify,
//       targets: value
//         .targets
//         .map(browserslist_to_lightningcss_targets)
//         .transpose()
//         .to_rspack_result_with_message(|e| format!("Failed to parse browserslist: {e}"))?
//         .flatten(),
//       error_recovery: value.error_recovery,
//       draft: value.draft.map(Into::into),
//       non_standard: value.non_standard.map(Into::into),
//       pseudo_classes: value.pseudo_classes.map(Into::into),
//       unused_symbols: value.unused_symbols,
//     })
//   }
// }

// impl From<RawDraft> for Draft {
//   fn from(value: RawDraft) -> Self {
//     Self {
//       custom_media: value.custom_media,
//       px_to_rem: value.px_to_rem.map(Into::into),
//     }
//   }
// }

// impl From<RawNonStandard> for NonStandard {
//   fn from(value: RawNonStandard) -> Self {
//     Self {
//       deep_selector_combinator: value.deep_selector_combinator,
//     }
//   }
// }

// impl From<RawPseudoClasses> for PseudoClasses {
//   fn from(value: RawPseudoClasses) -> Self {
//     Self {
//       hover: value.hover,
//       active: value.active,
//       focus: value.focus,
//       focus_visible: value.focus_visible,
//       focus_within: value.focus_within,
//     }
//   }
// }

// impl From<RawPxToRemOpts> for PxToRemOpts {
//   fn from(value: RawPxToRemOpts) -> Self {
//     Self {
//       root_value: value.root_value,
//       unit_precision: value.unit_precision,
//       prop_list: value.prop_list,
//       media_query: value.media_query,
//       min_pixel_value: value.min_pixel_value,
//     }
//   }
// }
