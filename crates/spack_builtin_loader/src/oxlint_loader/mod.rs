use std::sync::Arc;

use async_trait::async_trait;
use oxc::{
  allocator::Allocator,
  diagnostics::{GraphicalReportHandler, GraphicalTheme, NamedSource},
  parser::Parser,
  semantic::SemanticBuilder,
  span::SourceType,
};
use oxc_linter::{
  AllowWarnDeny, ConfigStore, ConfigStoreBuilder, ContextSubHost, ExternalPluginStore, FixKind,
  FrameworkFlags, LintOptions, Linter, Oxlintrc,
};
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifier;
use rspack_core::{Loader, LoaderContext, RunnerContext};
use rspack_error::Result;
use rspack_util::fx_hash::FxHashMap;
use serde::Serialize;
use serde_json::json;

pub const OXLINT_LOADER_IDENTIFIER: &str = "builtin:oxlint-loader";

#[cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct OxlintLoaderOpts {}

#[cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct OxlintLoader {
  options: OxlintLoaderOpts,
}

impl OxlintLoader {
  pub fn new(options: OxlintLoaderOpts) -> Self {
    Self { options }
  }

  pub fn get_config(&self) -> Oxlintrc {
    let config = json!({
      "plugins": ["eslint", "typescript", "unicorn", "react", "oxc"],
      "categories": {
        "correctness": "deny",
        "suspicious": "off",
        "pedantic": "off",
        "style": "off",
        "restriction": "off",
        "perf": "off",
        "nursery": "off"
      },
      "rules": {},
      "settings":{},
      "env":{},
      "globals":{},
      "overrides":[
      ],
      "ignorePatterns":[]
    });

    let config = serde_json::from_value::<Oxlintrc>(serde_json::to_value(config).unwrap()).unwrap();

    config
  }
}

#[async_trait]
#[cacheable_dyn]
impl Loader<RunnerContext> for OxlintLoader {
  fn identifier(&self) -> Identifier {
    OXLINT_LOADER_IDENTIFIER.into()
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let source = loader_context.take_content();
    let sm = loader_context.take_source_map();

    let Some(resource_path) = loader_context.resource_path().map(|p| p.to_path_buf()) else {
      return Ok(());
    };

    let Some(source_code) = source else {
      return Ok(());
    };

    let source_code = source_code.try_into_string()?;

    let config = self.get_config();

    let mut external_plugin_store = ExternalPluginStore::default();

    let config =
      ConfigStoreBuilder::from_oxlintrc(true, config.clone(), None, &mut external_plugin_store)
        .unwrap()
        .build(&external_plugin_store)
        .unwrap();

    let config_store = ConfigStore::new(config, FxHashMap::default(), external_plugin_store);

    let linter = Linter::new(
      LintOptions {
        fix: FixKind::None,
        framework_hints: FrameworkFlags::empty(),
        report_unused_directive: Some(AllowWarnDeny::Allow),
      },
      config_store,
      None,
    );

    let allocator = Allocator::default();

    let parser = Parser::new(&allocator, &source_code, SourceType::default());
    let parser_return = parser.parse();

    if parser_return.panicked {
      return Ok(());
    }

    let program = allocator.alloc(&parser_return.program);

    let semantic_builder_return = SemanticBuilder::new()
      .with_check_syntax_error(true)
      .with_cfg(true)
      .build(program);

    let semantic = semantic_builder_return.semantic;

    let module_record = Arc::new(oxc_linter::ModuleRecord::new(
      resource_path.as_std_path(),
      &parser_return.module_record,
      &semantic,
    ));

    let context_sub_hosts = ContextSubHost::new(semantic, module_record, 0);

    let messages = linter.run(
      resource_path.as_std_path(),
      vec![context_sub_hosts],
      &allocator,
    );

    if messages.is_empty() {
      loader_context.finish_with((source_code, sm));
      return Ok(());
    }

    // 配置带颜色和源代码上下文的 GraphicalReportHandler

    let handler = GraphicalReportHandler::new()
      .with_links(true)
      .with_theme(GraphicalTheme::unicode());

    let named_source = NamedSource::new(
      &resource_path.as_std_path().to_string_lossy().to_string(),
      source_code.to_string(),
    );

    // 将 lint 诊断信息推送到 rspack 的诊断系统
    for message in messages {
      let mut output = String::with_capacity(1024 * 1024);
      let diag = message.error.clone().with_source_code(named_source.clone());
      handler
        .render_report(&mut output, diag.as_ref())
        .map_err(|e| rspack_error::Error::from_error(e))?;
      eprintln!("{}", output);
      // loader_context
      //   .diagnostics
      //   .push();
    }

    loader_context.finish_with((source_code, sm));
    Ok(())
  }
}
