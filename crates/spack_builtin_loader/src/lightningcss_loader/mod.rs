use std::{
  borrow::Cow,
  collections::HashSet,
  sync::{Arc, RwLock},
};

use async_trait::async_trait;
use lightningcss::{
  printer::PrinterOptions,
  stylesheet::{MinifyOptions, ParserFlags, ParserOptions, StyleSheet},
  targets::{Features, Targets},
  visitor::Visit,
};
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifier;
use rspack_core::{
  Loader, LoaderContext, RunnerContext,
  rspack_sources::{Mapping, OriginalLocation, SourceMap, encode_mappings},
};
use rspack_error::{Result, ToStringResultToRspackResultExt};

use crate::{
  lightningcss_loader::{opts::LightningcssLoaderOpts, visitors::px_to_rem::PxToRemVisitor},
  loader_cache::LoaderWithIdentifier,
};

pub(crate) mod opts;
pub(crate) mod raws;
pub(crate) mod visitors;

pub const LIGHTNINGCSS_LOADER_IDENTIFIER: &str = "builtin:spack-lightningcss-loader";

#[cacheable]
#[derive(Clone)]
pub struct LightningcssLoader {
  identifier: Identifier,
  options: LightningcssLoaderOpts,
}

impl LightningcssLoader {
  pub fn new(options: LightningcssLoaderOpts) -> Self {
    Self {
      identifier: LIGHTNINGCSS_LOADER_IDENTIFIER.into(),
      options,
    }
  }

  fn get_flags(&self) -> ParserFlags {
    let mut flags = ParserFlags::empty();
    flags.set(
      ParserFlags::CUSTOM_MEDIA,
      matches!(&self.options.draft, Some(draft) if draft.custom_media),
    );
    flags.set(
      ParserFlags::DEEP_SELECTOR_COMBINATOR,
      matches!(&self.options.non_standard, Some(non_standard) if non_standard.deep_selector_combinator),
    );
    flags
  }
}

#[async_trait]
#[cacheable_dyn]
impl Loader<RunnerContext> for LightningcssLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(resource_path) = loader_context.resource_path() else {
      return Ok(());
    };

    let filename = resource_path.as_str().to_string();

    let Some(source) = loader_context.take_content() else {
      return Ok(());
    };

    let source_str = match &source {
      rspack_core::Content::String(s) => Cow::Borrowed(s.as_str()),
      rspack_core::Content::Buffer(buf) => String::from_utf8_lossy(buf),
    };

    let flags = self.get_flags();

    let error_recovery = self.options.error_recovery;

    let warnings = if error_recovery {
      Some(Arc::new(RwLock::new(Vec::new())))
    } else {
      None
    };

    let option = ParserOptions {
      filename: filename.clone(),
      css_modules: None,
      source_index: 0,
      error_recovery,
      warnings,
      flags,
    };

    let mut stylesheet = StyleSheet::parse(&source_str, option.clone()).to_rspack_result()?;

    // let mut stylesheet = to_static(
    //   stylesheet,
    //   ParserOptions {
    //     filename: filename.clone(),
    //     css_modules: None,
    //     source_index: 0,
    //     error_recovery: true,
    //     warnings: None,
    //     flags: ParserFlags::empty(),
    //   },
    // );

    // if let Some(visitors) = &self.visitors {
    //   let visitors = visitors.lock().await;
    //   for v in visitors.iter() {
    //     v(&mut stylesheet);
    //   }
    // }

    let targets = Targets {
      browsers: self.options.targets,
      include: Features::empty(),
      exclude: Features::empty(),
    };

    // let mut px_to_rem_replace = true;
    if let Some(draft) = &self.options.draft {
      if let Some(px_to_rem) = &draft.px_to_rem {
        // px_to_rem_replace = px_to_rem.replace;
        let mut px2rem = PxToRemVisitor::new(px_to_rem.clone());
        stylesheet.visit(&mut px2rem).unwrap();
      }
    };

    let unused_symbols = HashSet::<String>::new();

    // 只有在 px_to_rem replace 为 true 或者没有使用 px_to_rem 时才 minify
    // 因为 minify 会合并重复的属性，导致 fallback 被移除
    // if px_to_rem_replace {
    //   stylesheet
    //     .minify(MinifyOptions {
    //       targets,
    //       unused_symbols,
    //     })
    //     .to_rspack_result()?;
    // }

    stylesheet
      .minify(MinifyOptions {
        targets,
        unused_symbols,
        ..MinifyOptions::default()
      })
      .to_rspack_result()?;

    let mut parcel_source_map = if loader_context.context.source_map_kind.enabled() {
      Some(
        loader_context
          .source_map()
          .map(|input_source_map| -> Result<_> {
            let mut sm = parcel_sourcemap::SourceMap::new(
              input_source_map
                .source_root()
                .unwrap_or(&loader_context.context.options.context),
            );
            sm.add_source(&filename);
            sm.set_source_content(0, &source_str).to_rspack_result()?;
            Ok(sm)
          })
          .transpose()?
          .unwrap_or_else(|| {
            let mut source_map =
              parcel_sourcemap::SourceMap::new(&loader_context.context.options.context);
            let source_idx = source_map.add_source(&filename);
            source_map
              .set_source_content(source_idx as usize, &source_str)
              .expect("should set source content");
            source_map
          }),
      )
    } else {
      None
    };

    let pseudo_classes: Option<lightningcss::printer::PseudoClasses> = self
      .options
      .pseudo_classes
      .as_ref()
      .map(|item| lightningcss::printer::PseudoClasses {
        hover: item.hover.as_deref(),
        active: item.active.as_deref(),
        focus: item.focus.as_deref(),
        focus_visible: item.focus_visible.as_deref(),
        focus_within: item.focus_within.as_deref(),
      });

    let printer_options = PrinterOptions {
      minify: self.options.minify,
      source_map: parcel_source_map.as_mut(),
      project_root: None,
      targets,
      analyze_dependencies: None,
      pseudo_classes,
    };

    let content = stylesheet
      .to_css(printer_options)
      .to_rspack_result_with_message(|e| format!("failed to generate css: {e}"))?;

    println!("content--->{:#?}", content.code);

    if let Some(source_map) = parcel_source_map {
      let mappings = encode_mappings(source_map.get_mappings().iter().map(|mapping| Mapping {
        generated_line: mapping.generated_line,
        generated_column: mapping.generated_column,
        original: mapping.original.map(|original| OriginalLocation {
          source_index: original.source,
          original_line: original.original_line,
          original_column: original.original_column,
          name_index: original.name,
        }),
      }));
      let rspack_source_map = SourceMap::new(
        mappings,
        source_map
          .get_sources()
          .iter()
          .map(ToString::to_string)
          .collect::<Vec<_>>(),
        source_map
          .get_sources_content()
          .iter()
          .map(|source_content| Arc::from(source_content.to_string()))
          .collect::<Vec<_>>(),
        source_map
          .get_names()
          .iter()
          .map(ToString::to_string)
          .collect::<Vec<_>>(),
      );
      loader_context.finish_with((content.code, rspack_source_map));
    } else {
      loader_context.finish_with(content.code);
    }

    Ok(())
  }
}

impl LoaderWithIdentifier for LightningcssLoader {
  fn with_identifier(mut self, identifier: Identifier) -> Self {
    assert!(identifier.starts_with(LIGHTNINGCSS_LOADER_IDENTIFIER));
    self.identifier = identifier;
    self
  }
}
