use std::{
  panic::{AssertUnwindSafe, catch_unwind},
  path::Path,
  sync::Arc,
};

use oxc::{
  allocator::Allocator,
  diagnostics::{GraphicalReportHandler, GraphicalTheme, NamedSource},
  parser::Parser,
  semantic::SemanticBuilder,
  span::SourceType,
};
use oxc_linter::{
  AllowWarnDeny, ConfigStore, ConfigStoreBuilder, ContextSubHost, ExternalPluginStore, FixKind,
  FrameworkFlags, LintOptions, Linter, Message, Oxlintrc,
};
use rspack_error::Result;
use rustc_hash::FxHashMap;

#[derive(Debug)]
pub struct LintRunner {
  linter: Arc<Linter>,
  handler: Arc<GraphicalReportHandler>,
  show_warning: bool,
}

impl LintRunner {
  pub fn new(oxlintrc: Oxlintrc, show_warning: bool) -> Self {
    // 构建 linter
    let mut external_plugin_store = ExternalPluginStore::default();
    let config =
      ConfigStoreBuilder::from_oxlintrc(true, oxlintrc.clone(), None, &mut external_plugin_store)
        .expect("Failed to build inner oxlintrc config store builder.")
        .build(&external_plugin_store)
        .expect("Failed to build oxlintrc config.");

    let linter = Arc::new(Linter::new(
      LintOptions {
        fix: FixKind::None,
        framework_hints: FrameworkFlags::React,
        report_unused_directive: Some(AllowWarnDeny::Deny),
      },
      ConfigStore::new(config, FxHashMap::default(), external_plugin_store),
      None,
    ));

    // 构建 handler
    let handler = Arc::new(
      GraphicalReportHandler::new()
        .with_links(true)
        .with_link_display_text("View in editor")
        .with_theme(GraphicalTheme::unicode()),
    );

    Self {
      linter,
      handler,
      show_warning,
    }
  }

  pub async fn lint(&self, resource: impl AsRef<Path>) -> Result<Vec<Message>> {
    let path = resource.as_ref();

    let allocator = Allocator::default();

    let source_type =
      SourceType::from_path(&path).map_err(|e| rspack_error::Error::from_error(e))?;

    let source_code = tokio::fs::read_to_string(path).await?;

    let parse_options = oxc::parser::ParseOptions {
      parse_regular_expression: true,
      allow_return_outside_function: false,
      preserve_parens: true,
      allow_v8_intrinsics: false,
    };

    let parser = Parser::new(&allocator, &source_code, source_type).with_options(parse_options);

    let parser_return = parser.parse();

    if parser_return.panicked {
      eprintln!("Warning: Failed to parse file: {:?}", path);
      return Ok(vec![]);
    }

    let program = allocator.alloc(&parser_return.program);

    let semantic_builder_return = SemanticBuilder::new()
      .with_check_syntax_error(true)
      .with_cfg(true)
      .build(program);

    let semantic = semantic_builder_return.semantic;

    let module_record = Arc::new(oxc_linter::ModuleRecord::new(
      path,
      &parser_return.module_record,
      &semantic,
    ));

    let context_sub_hosts = ContextSubHost::new(semantic, module_record, 0);

    let result = catch_unwind(AssertUnwindSafe(|| {
      self
        .linter
        .run_with_disable_directives(path, vec![context_sub_hosts], &allocator)
    }));

    let (messages, _disable_directives) = match result {
      Ok(result) => result,
      Err(e) => {
        eprintln!(
          "Warning: Failed to process disable directives for {:?}, falling back to basic linting: {:?}",
          &path, e,
        );
        (vec![], None)
      }
    };

    let named_source = NamedSource::new(path.to_string_lossy(), source_code.clone());

    if messages.is_empty() {
      return Ok(vec![]);
    }

    for message in messages.clone() {
      let error = message.error;

      let show = match error.severity {
        oxc::diagnostics::Severity::Error => true,
        oxc::diagnostics::Severity::Warning | oxc::diagnostics::Severity::Advice => {
          self.show_warning
        }
      };

      if !show {
        continue;
      }

      let mut output = String::with_capacity(128);

      let report = error.with_source_code(named_source.clone());
      self
        .handler
        .render_report(&mut output, report.as_ref())
        .map_err(|e| rspack_error::Error::from_error(e))?;

      eprintln!("{}", output);
    }

    return Ok(messages);
  }
}
