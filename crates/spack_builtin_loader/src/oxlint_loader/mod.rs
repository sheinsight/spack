use std::{collections::HashMap, ops::Not, sync::Arc};

use async_trait::async_trait;
use num_format::{Locale, ToFormattedString};
use owo_colors::OwoColorize;
use oxc::{
  allocator::Allocator,
  diagnostics::{
    GraphicalReportHandler, GraphicalTheme, NamedSource, OxcCode, OxcDiagnostic, Severity,
  },
  parser::Parser,
  semantic::SemanticBuilder,
  span::SourceType,
};
use oxc_linter::{
  AllowWarnDeny, ConfigStore, ConfigStoreBuilder, ContextSubHost, DisableDirectives,
  ExternalPluginStore, FixKind, FrameworkFlags, LintOptions, Linter, Message, Oxlintrc,
};
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifier;
use rspack_core::{Loader, LoaderContext, RunnerContext};
use rspack_error::Result;
use rspack_paths::Utf8PathBuf;
use rspack_util::fx_hash::FxHashMap;
use serde::Serialize;
use serde_json::json;

use crate::oxlint_loader::environments::Environment;
use crate::oxlint_loader::restricted::Restricted;

pub mod environments;
pub mod restricted;

pub const OXLINT_LOADER_IDENTIFIER: &str = "builtin:oxlint-loader";

#[cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct OxLintLoaderOpts {
  pub output_dir: String,
  pub show_warning: bool,
  pub restricted_imports: Vec<Restricted>,
  pub restricted_globals: Vec<Restricted>,
  pub globals: HashMap<String, bool>,
  pub environments: Environment,
}

#[cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct OxLintLoader {
  options: OxLintLoaderOpts,
}

impl OxLintLoader {
  pub fn new(options: OxLintLoaderOpts) -> Self {
    Self { options }
  }

  pub fn write_runtime(&self, dir: &Utf8PathBuf) -> Result<()> {
    if dir.exists().not() {
      std::fs::create_dir_all(dir)?;
    }

    let file = dir.join(".oxlintrc.json");

    let config = self
      .get_config()
      .map_err(|e| rspack_error::Error::from_error(e))?;

    std::fs::write(
      file,
      serde_json::to_string_pretty(&config).map_err(|e| rspack_error::Error::from_error(e))?,
    )?;

    Ok(())
  }

  fn get_config(&self) -> serde_json::Result<serde_json::Value> {
    let restricted_imports = serde_json::to_value(&self.options.restricted_imports)?;
    let restricted_globals = serde_json::to_value(&self.options.restricted_globals)?;

    let globals = serde_json::to_value(&self.options.globals)?;

    let environments = serde_json::to_value(&self.options.environments)?;

    let config = json!({
      "plugins": [
        "eslint",
        "typescript",
        "unicorn",
        "react",
        "oxc"
      ],
      "categories": {
        "correctness": "off",
        "suspicious": "off",
        "pedantic": "off",
        "style": "off",
        "restriction": "off",
        "perf": "off",
        "nursery": "off"
      },
      "rules": {
        "eslint/for-direction":[2],
        "eslint/no-async-promise-executor":[2],
        "eslint/no-caller":[0],
        "eslint/no-class-assign":[2],
        "eslint/no-compare-neg-zero":[2],
        "eslint/no-cond-assign":[2],
        "eslint/no-const-assign":[2],
        "eslint/no-constant-binary-expression":[2],
        "eslint/no-constant-condition":[0],
        "eslint/no-control-regex":[2],
        "eslint/no-debugger":[0],
        "eslint/no-delete-var":[2],
        "eslint/no-dupe-class-members":[2],
        "eslint/no-dupe-else-if":[2],
        "eslint/no-dupe-keys":[2],
        "eslint/no-duplicate-case":[2],
        "eslint/no-empty-character-class":[2],
        "eslint/no-empty-pattern":[2],
        "eslint/no-empty-static-block":[2],
        "eslint/no-eval":[1,{
          "allowIndirect": true,
        }],
        "eslint/no-ex-assign":[2],
        "eslint/no-extra-boolean-cast":[1],
        "eslint/no-func-assign":[2],
        "eslint/no-global-assign":[2],
        "eslint/no-import-assign":[2],
        "eslint/no-invalid-regexp":[2],
        "eslint/no-irregular-whitespace":[2],
        "eslint/no-loss-of-precision":[2],
        "eslint/no-new-native-nonconstructor":[2],
        "eslint/no-nonoctal-decimal-escape":[2],
        "eslint/no-obj-calls":[2],
        "eslint/no-self-assign":[0],
        "eslint/no-setter-return":[2],
        "eslint/no-shadow-restricted-names":[2],
        "eslint/no-sparse-arrays":[0],
        "eslint/no-this-before-super":[2],
        "eslint/no-unassigned-vars":[0],
        "eslint/no-unsafe-finally":[2],
        "eslint/no-unsafe-negation":[2,{
          "enforceForOrderingRelations":false
        }],
        "eslint/no-unsafe-optional-chaining":[2],
        "eslint/no-unused-expressions":[0],
        "eslint/no-unused-labels":[0],
        "eslint/no-unused-private-class-members":[0],
        "eslint/no-unused-vars":[0],
        "eslint/no-useless-backreference":[2],
        "eslint/no-useless-catch":[2],
        "eslint/no-useless-escape":[1],
        "eslint/no-useless-rename":[2],
        "eslint/no-with":[0],
        "eslint/require-yield":[2],
        "eslint/use-isnan":[2],
        "eslint/valid-typeof":[2],
        "eslint/no-await-in-loop":[2],
        "eslint/no-useless-call":[0],
        "eslint/class-methods-use-this":[0],
        "eslint/default-case":[0],
        "eslint/no-alert":[1],
        "eslint/no-bitwise":[0],
        "eslint/no-console":[1],
        "eslint/no-div-regex":[1],
        "eslint/no-empty":[1],
        "eslint/no-empty-function":[1,{"allow":["constructors","arrowFunctions"]}],
        "eslint/no-eq-null":[2],
        "eslint/no-iterator":[2],
        "eslint/no-param-reassign":[2],
        "eslint/no-plusplus":[0],
        "eslint/no-proto":[2],
        "eslint/no-regex-spaces":[2],
        // TODO: 添加 no-restricted-globals 规则
        "no-restricted-globals": [2, restricted_globals],
        // TODO: 添加 restricted-imports 规则
        "no-restricted-imports": [2, {
          "paths": restricted_imports
        }],
        "eslint/no-undefined":[1],
        "eslint/no-var":[2],
        "eslint/no-void":[0],
        "eslint/unicode-bom":[2],
        "eslint/block-scoped-var":[2],
        "eslint/no-extend-native":[2],
        "eslint/no-extra-bind":[2],
        "eslint/no-new":[1],
        "eslint/no-unexpected-multiline":[2],
        "eslint/no-unneeded-ternary":[1],
        "eslint/no-useless-concat":[2],
        "eslint/no-useless-constructor":[0],
        "eslint/preserve-caught-error":[2,{
          "requireCatchParameter":false
        }],
        "eslint/array-callback-return":[1],
        "eqeqeq": [2, "always", {
          "null": "always"
        }],
        "eslint/max-classes-per-file":[2,{
          "max":1
        }],
        "eslint/max-depth":[0],
        "eslint/max-lines":[2,{
          "max":1000,
          "skipBlankLines":true,
          "skipComments":true
        }],
        "eslint/max-lines-per-function":[2,{
          "max": 300,
          "skipBlankLines": true,
          "skipComments": true,
          "IIFEs": false
        }],
        "eslint/max-nested-callbacks":[2,{
          "max": 9
        }],
        "eslint/no-array-constructor":[2],
        "eslint/no-case-declarations":[2],
        "eslint/no-constructor-return":[2],
        "eslint/no-else-return":[0],
        "eslint/no-fallthrough":[0],
        "eslint/no-inner-declarations":[0],
        "eslint/no-lonely-if":[1],
        "eslint/no-negated-condition":[0],
        "eslint/no-new-wrappers":[2],
        "eslint/no-object-constructor":[2],
        "eslint/no-prototype-builtins":[2],
        "eslint/no-redeclare":[2,{"builtinGlobals":true}],
        "eslint/no-self-compare":[2],
        "eslint/no-throw-literal":[2],
        "eslint/radix":[0],
        "eslint/require-await":[2],
        "eslint/sort-vars":[0],
        "eslint/symbol-description":[2],
        "eslint/getter-return":[2],
        "eslint/no-misleading-character-class":[0],
        "eslint/no-undef":[2],
        "eslint/no-unreachable":[2]
      },
      "settings":{},
      "env":environments,
      "globals": globals,
      "overrides":[{
        "files": ["*.{ts,tsx,cts,mts}"],
        "env": {},
        "globals": {},
        "plugins": [],
        "rules":{}
      },{
        "files": ["*.{jsx,tsx}"],
        "env": {},
        "globals": {},
        "plugins": [],
        "rules":{}
      }],
      "ignorePatterns":[]
    });

    Ok(config)
  }
}

impl OxLintLoader {
  fn create_report(
    &self,
    named_source: &NamedSource<String>,
    message: &Message,
  ) -> oxc::diagnostics::Error {
    let msg = &message.error;

    let message_text = msg.message.to_string();

    // 使用引用解构，避免 clone
    let OxcCode { number, .. } = &msg.code;
    let number = number.clone(); // 只 clone number 字段
    let error = match msg.severity {
      Severity::Error => OxcDiagnostic::error(message_text.clone()),
      Severity::Warning | Severity::Advice => OxcDiagnostic::warn(message_text.clone()),
    };

    // 直接使用引用
    let mut error = error.with_error_code("LEGO", number.unwrap_or_else(|| "Unknown".into()));

    if let Some(labels) = &msg.labels {
      error = error.with_labels(labels.iter().cloned());
    }

    if let Some(help) = &msg.help {
      error = error.with_help(help.to_string());
    }

    // 如果 API 允许，考虑用 Arc 包装 named_source 避免循环中 clone
    error.with_source_code(named_source.clone())
  }

  fn lint(
    &self,
    source_code: &str,
    resource_path: &Utf8PathBuf,
  ) -> Result<(Vec<Message>, Option<DisableDirectives>)> {
    let config = self
      .get_config()
      .map_err(|e| rspack_error::Error::from_error(e))?;

    let config =
      serde_json::from_value::<Oxlintrc>(config).map_err(|e| rspack_error::Error::from_error(e))?;

    let mut external_plugin_store = ExternalPluginStore::default();

    let config =
      ConfigStoreBuilder::from_oxlintrc(true, config.clone(), None, &mut external_plugin_store)
        .unwrap()
        .build(&external_plugin_store)
        .map_err(|e| rspack_error::Error::from_error(e))?;

    let config_store = ConfigStore::new(config, FxHashMap::default(), external_plugin_store);

    let linter = Linter::new(
      LintOptions {
        fix: FixKind::None,
        framework_hints: FrameworkFlags::empty(),
        report_unused_directive: Some(AllowWarnDeny::Deny),
      },
      config_store,
      None,
    );

    let allocator = Allocator::default();

    let parser = Parser::new(&allocator, &source_code, SourceType::default());
    let parser_return = parser.parse();

    if parser_return.panicked {
      return Ok((vec![], None));
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

    let (messages, disable_directives) = linter.run_with_disable_directives(
      resource_path.as_std_path(),
      vec![context_sub_hosts],
      &allocator,
    );

    let messages = messages
      .into_iter()
      .filter(|message| match message.error.severity {
        Severity::Error => true,
        _ => self.options.show_warning,
      })
      .collect();

    Ok((messages, disable_directives))
  }

  fn print_message_diagnostics(
    &self,
    resource_path: &Utf8PathBuf,
    source_code: &str,
    messages: &Vec<Message>,
  ) -> Result<()> {
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
      let error = self.create_report(&named_source, &message);
      handler
        .render_report(&mut output, error.as_ref())
        .map_err(|e| rspack_error::Error::from_error(e))?;
      eprintln!("{}", output);
    }

    Ok(())
  }

  fn print_disable_directives_info(&self, disable_directives: &DisableDirectives) -> Result<()> {
    // 分组存储每个规则的所有出现位置
    // let mut rule_spans: FxHashMap<String, Vec<DisableRuleComment>> = FxHashMap::default();

    // for comment in disable_directives.disable_rule_comments() {
    //   match &comment.r#type {
    //     RuleCommentType::All => {
    //       rule_spans
    //         .entry("__ALL__".to_string())
    //         .or_insert_with(Vec::new)
    //         .push(comment.clone());
    //     }
    //     RuleCommentType::Single(rules) => {
    //       for rule in rules {
    //         rule_spans
    //           .entry(rule.rule_name.to_string())
    //           .or_insert_with(Vec::new)
    //           .push(comment.clone());
    //       }
    //     }
    //   };
    // }

    let len = disable_directives.disable_rule_comments().len();

    if len > 0 {
      let len = len.to_formatted_string(&Locale::en);

      eprintln!(
        r##"
{:<4}{} times have you disable the eslint rules. 

Though it be a compromise wrought by the moment, I hold faith that you shall, in the fullness of time, emerge unbound.{:>3}
"##,
        "⚔️",
        len.red().bold(),
        "✨"
      );
    }
    Ok(())
  }

  // 创建美化的 disable directives 诊断输出
  // fn print_disable_directives_info(
  //   &self,
  //   disable_directives: &Option<oxc_linter::DisableDirectives>,
  //   source_code: &str,
  // ) -> Result<()> {
  //   // 如果没有 disable directives,直接返回
  //   let Some(directives) = disable_directives else {
  //     return Ok(());
  //   };
  //   let handler = GraphicalReportHandler::new()
  //     .with_links(true)
  //     .with_theme(GraphicalTheme::unicode());

  //   // 统计信息
  //   let disable_rule_count = directives.disable_rule_comments().len();
  //   let unused_enable_count = directives.unused_enable_comments().len();

  //   // 创建主诊断信息
  //   let mut diagnostic = OxcDiagnostic::warn("Disable Directives Analysis").with_help(format!(
  //     "Found {} disable-rule comments, {} unused-enable comments",
  //     disable_rule_count, unused_enable_count
  //   ));

  //   // 添加 disable-rule 注释的标签
  //   for comment in directives.disable_rule_comments().iter().take(5) {
  //     use oxc_linter::RuleCommentType;
  //     let label_text = match &comment.r#type {
  //       RuleCommentType::All => "disable all rules".to_string(),
  //       RuleCommentType::Single(rules) => {
  //         let rules_text = rules
  //           .iter()
  //           .map(|r| r.rule_name.as_str())
  //           .collect::<Vec<_>>()
  //           .join(", ");
  //         format!("disable: {}", rules_text)
  //       }
  //     };
  //     diagnostic =
  //       diagnostic.with_label(oxc::diagnostics::LabeledSpan::at(comment.span, label_text));
  //   }

  //   // 添加未使用的 enable 注释的标签
  //   for (rule_name, span) in directives.unused_enable_comments().iter().take(5) {
  //     let label_text = if let Some(name) = rule_name {
  //       format!("unused enable: {}", name)
  //     } else {
  //       "unused enable (all)".to_string()
  //     };
  //     diagnostic = diagnostic.with_label(oxc::diagnostics::LabeledSpan::at(*span, label_text));
  //   }

  //   // 如果标签过多，添加省略提示
  //   let total_labels = disable_rule_count + unused_enable_count;
  //   if total_labels > 5 {
  //     diagnostic = diagnostic.with_help(format!(
  //       "Showing first 5 of {} total directives. Use detailed logging for full list.",
  //       total_labels
  //     ));
  //   }

  //   let named_source = NamedSource::new("disable_directives", source_code.to_string());
  //   let diagnostic = diagnostic.with_source_code(named_source);

  //   // 渲染并输出
  //   let mut output = String::with_capacity(4096);
  //   handler
  //     .render_report(&mut output, diagnostic.as_ref())
  //     .map_err(|e| rspack_error::Error::from_error(e))?;

  //   eprintln!("{}", output);
  //   Ok(())
  // }
}

#[async_trait]
#[cacheable_dyn]
impl Loader<RunnerContext> for OxLintLoader {
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

    let (messages, disable_directives) = self.lint(&source_code, &resource_path)?;

    let has_messages = !messages.is_empty();

    if has_messages {
      self.print_message_diagnostics(&resource_path, &source_code, &messages)?;
    }

    if has_messages {
      for message in messages {
        let message_text = message.error.message.to_string();

        let error = match message.error.severity {
          Severity::Error => rspack_error::Error::error(message_text),
          _ => rspack_error::Error::warning(message_text),
        };

        loader_context
          .diagnostics
          .push(rspack_error::Diagnostic::from(error));
      }
    }

    if let Some(disable_directives) = disable_directives {
      self.print_disable_directives_info(&disable_directives)?;
    }

    loader_context.finish_with((source_code, sm));
    Ok(())
  }
}
