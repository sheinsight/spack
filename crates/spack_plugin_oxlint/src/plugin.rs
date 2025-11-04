use std::{
  collections::HashMap,
  panic::{AssertUnwindSafe, catch_unwind},
  path::Path,
  sync::{Arc, Mutex},
};

use ignore::WalkBuilder;
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
use rspack_core::{Compilation, CompilationParams, Plugin};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxHashMap, FxHashSet};
use serde_json::json;

use crate::{Environment, Restricted};

#[derive(Debug, Clone)]
pub struct OxlintPluginOpts {
  pub output_dir: String,
  pub show_warning: bool,
  pub fail_on_error: bool,
  pub restricted_imports: Vec<Restricted>,
  pub restricted_globals: Vec<Restricted>,
  pub globals: HashMap<String, bool>,
  pub environments: Environment,
  pub oxlint_config_file_path: Option<String>,
}

pub const OX_LINT_PLUGIN_IDENTIFIER: &'static str = "Spack.OxlintPlugin";

#[plugin]
#[derive(Debug)]
pub struct OxlintPlugin {
  #[allow(unused)]
  options: OxlintPluginOpts,
  linter: Arc<Linter>,
  handler: Arc<GraphicalReportHandler>,
  cache: Arc<Mutex<FxHashMap<String, Vec<Message>>>>,
  linted_files: Arc<Mutex<FxHashSet<String>>>,
  initialized: Arc<Mutex<bool>>,
}

impl OxlintPlugin {
  pub fn new(options: OxlintPluginOpts) -> Self {
    // 1. 构建配置
    let config = Self::get_oxlintrc(&options);

    // 3. 构建 linter
    let mut external_plugin_store = ExternalPluginStore::default();
    let config =
      ConfigStoreBuilder::from_oxlintrc(true, config.clone(), None, &mut external_plugin_store)
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

    // 4. 构建 handler
    let handler = Arc::new(
      GraphicalReportHandler::new()
        .with_links(true)
        .with_link_display_text("View in editor")
        .with_theme(GraphicalTheme::unicode()),
    );

    let cache = Arc::new(Mutex::new(FxHashMap::default()));

    let linted_files = Arc::new(Mutex::new(FxHashSet::default()));

    Self::new_inner(
      options,
      linter,
      handler,
      cache,
      linted_files,
      Arc::new(Mutex::new(false)),
    )
  }
}

impl OxlintPlugin {
  fn build_config(options: &OxlintPluginOpts) -> serde_json::Result<serde_json::Value> {
    let restricted_imports = serde_json::to_value(&options.restricted_imports)?;
    let restricted_globals = serde_json::to_value(&options.restricted_globals)?;

    let globals = serde_json::to_value(&options.globals)?;

    let environments = serde_json::to_value(&options.environments)?;

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
        // eslint rules
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
        "eslint/no-console":[0],
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
        "eslint/no-undefined":[0],
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
        "eslint/max-classes-per-file":[1,{
          "max":1,
          "skipBlankLines":false,
          "skipComments":false
        }],
        "eslint/max-depth":[0],
        "eslint/max-lines":[1,{
          "max":1000,
          "skipBlankLines":false,
          "skipComments":false
        }],
        "eslint/max-lines-per-function":[1,{
          "max": 300,
          "skipBlankLines": false,
          "skipComments": false,
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
        "eslint/no-unreachable":[2],
        // oxc rules
        "oxc/bad-array-method-on-arguments":[2],
        "oxc/bad-char-at-comparison":[2],
        "oxc/bad-comparison-sequence":[2],
        "oxc/bad-min-max-func":[1],
        "oxc/bad-object-literal-comparison":[2],
        "oxc/bad-replace-all-arg":[2],
        "oxc/const-comparisons":[2],
        "oxc/double-comparisons":[1],
        "oxc/erasing-op":[1],
        "oxc/missing-throw":[1],
        "oxc/number-arg-out-of-range":[2],
        "oxc/only-used-in-recursion":[1],
        "oxc/uninvoked-array-callback":[2],
        "oxc/no-accumulating-spread":[1],
        "oxc/bad-bitwise-operator":[0],
        "oxc/no-async-await":[0],
        "oxc/no-barrel-file":[1],
        "oxc/no-const-enum":[2],
        "oxc/no-optional-chaining":[0],
        "oxc/no-rest-spread-properties":[0],
        "oxc/approx-constant":[1],
        "oxc/misrefactored-assign-op":[1],
        "oxc/no-async-endpoint-handlers":[0],
        "oxc/branches-sharing-code":[1],
        "oxc/no-map-spread":[0],
        // promise
        "promise/no-callback-in-promise":[0],
        "promise/no-new-statics":[2],
        "promise/valid-params":[2],
        "promise/catch-or-return":[2],
        "promise/spec-only":[2],
        "promise/always-return":[2],
        "promise/no-multiple-resolved":[2],
        "promise/no-promise-in-callback":[1],
        "promise/avoid-new":[1],
        "promise/no-nesting":[1],
        "promise/no-return-wrap":[1],
        "promise/param-names":[1],
        "promise/prefer-await-to-callbacks":[0],
        "promise/prefer-await-to-then":[1],
        "promise/prefer-catch":[1],
        "promise/no-return-in-finally":[2],
        // unicorn
        // "unicorn/no-await-in-promise-methods":[2],
        // "unicorn/no-empty-file":[1],
        // "unicorn/no-invalid-fetch-options":[2],
        // "unicorn/no-invalid-remove-event-listener":[2],
        // "unicorn/no-new-array":[1],
        // "unicorn/no-single-promise-in-promise-methods":[1],
        // "unicorn/no-thenable":[1],
        // "unicorn/no-unnecessary-await":[2],
        // "unicorn/no-useless-fallback-in-spread":[1],
        // "unicorn/no-useless-length-check":[1],
        // "unicorn/no-useless-spread":[1],
        // "unicorn/prefer-set-size":[1],
        // "unicorn/prefer-string-starts-ends-with":[1],
        // "unicorn/prefer-array-find":[1],
        // "unicorn/prefer-array-flat-map":[1],
        // "unicorn/prefer-set-has":[0],
        // "unicorn/no-abusive-eslint-disable":[2],
        // "unicorn/no-anonymous-default-export":[2],
        // "unicorn/no-array-for-each":[1],
        // "unicorn/no-array-reduce":[0],
        // "unicorn/no-document-cookie":[2],
        // "unicorn/no-length-as-slice-end":[1],
        // "unicorn/no-magic-array-flat-depth":[0],
        // "unicorn/no-process-exit":[0],
        // "unicorn/no-useless-error-capture-stack-trace":[2],
        // "unicorn/prefer-modern-math-apis":[1],
        // "unicorn/prefer-node-protocol":[2],
        // "unicorn/prefer-number-properties":[0],
        // "unicorn/consistent-function-scoping":[0],
        // "unicorn/no-accessor-recursion":[2],
        // "unicorn/no-array-reverse":[1,{"allowExpressionStatement":true}],
        // "unicorn/no-array-sort":[1,{"allowExpressionStatement":true}],
        // "unicorn/no-instanceof-builtins":[1],
        // "unicorn/prefer-add-event-listener":[1],
        // "unicorn/require-module-specifiers":[1],
        // "unicorn/require-post-message-target-origin":[2],
        // "unicorn/consistent-assert":[0],
        // "unicorn/consistent-empty-array-spread":[1],
        // "unicorn/escape-case":[1],
        // "unicorn/explicit-length-check":[0],
        // "unicorn/new-for-builtins":[2],
        // "unicorn/no-array-callback-reference":[0],
        // "unicorn/no-hex-escape":[0],
        // "unicorn/no-instanceof-array":[2],
        // "unicorn/no-lonely-if":[1],
        // "unicorn/no-negation-in-equality-check":[1]
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

  fn build_overrides(
    dir: impl AsRef<Path>,
  ) -> std::result::Result<ignore::overrides::Override, ignore::Error> {
    let mut overrides = ignore::overrides::OverrideBuilder::new(&dir);

    // 包含特定扩展名（分别添加）
    overrides.add("*.js")?;
    overrides.add("*.jsx")?;
    overrides.add("*.ts")?;
    overrides.add("*.tsx")?;
    overrides.add("*.mjs")?;
    overrides.add("*.cjs")?;
    overrides.add("*.cts")?;
    overrides.add("*.mts")?;

    // 排除特定文件
    overrides.add("!*.d.ts")?;
    overrides.add("!*.min.js")?;

    // 排除目录
    overrides.add("!node_modules/**")?;
    overrides.add("!**/.lego/**")?;
    overrides.add("!**/node_modules/**")?;
    overrides.add("!dist/**")?;
    overrides.add("!build/**")?;
    overrides.add("!coverage/**")?;
    overrides.add("!.git/**")?;

    let overrides = overrides.build()?;

    Ok(overrides)
  }

  fn get_oxlintrc(options: &OxlintPluginOpts) -> Oxlintrc {
    let config = if let Some(oxlintrc_file_path) = &options.oxlint_config_file_path {
      Oxlintrc::from_file(Path::new(oxlintrc_file_path)).expect(&format!(
        "Failed to load oxlintrc file: {}",
        oxlintrc_file_path
      ))
    } else {
      let config = Self::build_config(options).expect("Failed to build inner oxlintrc config.");

      let pretty_config =
        serde_json::to_string_pretty(&config).expect("Failed to pretty print oxlintrc config.");

      let config_output_file_path = Path::new(&options.output_dir).join(".oxlintrc.json");

      if !Path::new(&options.output_dir).exists() {
        std::fs::create_dir_all(&options.output_dir).expect(&format!(
          "Failed to create output directory: {:?}",
          &options.output_dir
        ));
      }

      std::fs::write(&config_output_file_path, pretty_config).expect(&format!(
        "Failed to write oxlintrc file to: {:?}",
        config_output_file_path
      ));

      serde_json::from_value::<Oxlintrc>(config).expect(&format!(
        "Failed to build inner oxlintrc config store builder."
      ))
    };
    config
  }

  async fn lint(&self, resource: impl AsRef<Path>) -> Result<Vec<Message>> {
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

    let mut _error_count = 0;
    let mut _warning_count = 0;

    for message in messages.clone() {
      let error = message.error;

      let show = match error.severity {
        oxc::diagnostics::Severity::Error => {
          _error_count += 1;
          true
        }
        oxc::diagnostics::Severity::Warning | oxc::diagnostics::Severity::Advice => {
          _warning_count += 1;
          self.options.show_warning
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

impl Plugin for OxlintPlugin {
  fn name(&self) -> &'static str {
    OX_LINT_PLUGIN_IDENTIFIER.into()
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext) -> Result<()> {
    ctx
      .compilation_hooks
      .succeed_module
      .tap(succeed_module::new(self));

    ctx
      .compiler_hooks
      .this_compilation
      .tap(this_compilation::new(self));

    Ok(())
  }

  fn clear_cache(&self, _id: rspack_core::CompilationId) {}
}

#[plugin_hook(rspack_core::CompilerThisCompilation for OxlintPlugin)]
pub(crate) async fn this_compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  // 检查是否是首次启动
  let is_first_run = if let Ok(mut initialized) = self.initialized.lock() {
    if !*initialized {
      *initialized = true;
      true
    } else {
      false
    }
  } else {
    return Ok(());
  };

  // 每次 this_compilation 开始时，清空 linted_files（标记当前编译周期）
  // 这样后续热更新时，succeed_module 中的文件可以正常 lint
  if let Ok(mut linted_files) = self.linted_files.lock() {
    linted_files.clear();
  }

  // 只在首次启动时执行全量 lint
  if !is_first_run {
    // 后续热更新时，只更新 diagnostics，不执行全量 lint
    let error_count = self
      .cache
      .lock()
      .map(|c| {
        c.values()
          .flatten()
          .filter(|m| m.error.severity == oxc::diagnostics::Severity::Error)
          .count()
      })
      .unwrap_or(0);

    let diagnostics = compilation.diagnostics_mut();
    diagnostics.push(Diagnostic::error(
      OX_LINT_PLUGIN_IDENTIFIER.into(),
      format!("Lint errors in total: {}", error_count),
    ));

    return Ok(());
  }

  // 首次启动：执行全量 lint
  let context = compilation.options.context.as_path();

  let overrides = Self::build_overrides(context).expect("Failed to build ignore overrides.");

  let files = WalkBuilder::new(context)
    .overrides(overrides)
    .build()
    .filter_map(|e| e.ok())
    .filter(|e| e.file_type().map_or(false, |ft| ft.is_file()))
    .map(|e| e.path().to_owned())
    .collect::<Vec<_>>();

  // 记录所有 lint 过的文件（首次启动时，避免 succeed_module 重复 lint）
  if let Ok(mut linted_files) = self.linted_files.lock() {
    for file in &files {
      let resource = file.to_string_lossy().into_owned();
      linted_files.insert(resource);
    }
  }

  for file in files {
    let resource = file.to_string_lossy().into_owned();

    let messages = self.lint(&file).await?;

    if !messages.is_empty() {
      if let Ok(mut cache) = self.cache.lock() {
        cache.insert(resource, messages);
      };
    }
  }

  let error_count = self
    .cache
    .lock()
    .map(|c| {
      c.values()
        .flatten()
        .filter(|m| m.error.severity == oxc::diagnostics::Severity::Error)
        .count()
    })
    .unwrap_or(0);

  let diagnostics = compilation.diagnostics_mut();

  diagnostics.push(Diagnostic::error(
    OX_LINT_PLUGIN_IDENTIFIER.into(),
    format!("Lint errors in total: {}", error_count),
  ));

  if error_count > 0 && !compilation.options.mode.is_development() && self.options.fail_on_error {
    return Err(rspack_error::Error::error(format!(
      "Lint errors in total: {}",
      error_count
    )));
  }

  Ok(())
}

#[plugin_hook(rspack_core::CompilationSucceedModule for OxlintPlugin)]
pub(crate) async fn succeed_module(
  &self,
  _compiler_id: rspack_core::CompilerId,
  _compilation_id: rspack_core::CompilationId,
  module: &mut rspack_core::BoxModule,
) -> Result<()> {
  let Some(normal_module) = module.as_normal_module() else {
    return Ok(());
  };

  let resource = normal_module.resource_resolved_data().resource();

  let overrides = Self::build_overrides("/").expect("Failed to build ignore overrides.");

  let matcher = overrides.matched(resource, false);

  if !matcher.is_whitelist() {
    return Ok(());
  }

  // 检查文件是否在当前编译周期中已经 lint 过
  // 首次启动时：如果文件在全量 lint 中处理过，跳过（避免重复）
  // 后续热更新时：linted_files 已在 this_compilation 中清空，所以会正常 lint
  let should_lint = if let Ok(linted_files) = self.linted_files.lock() {
    !linted_files.contains(resource)
  } else {
    true
  };

  if should_lint {
    // 执行 lint
    let messages = self.lint(resource).await?;

    // 标记为已 lint（避免同一个编译周期内重复 lint）
    if let Ok(mut linted_files) = self.linted_files.lock() {
      linted_files.insert(resource.to_string());
    }

    // 更新 cache
    if !messages.is_empty() {
      if let Ok(mut cache) = self.cache.lock() {
        cache.insert(resource.to_string(), messages);
      }
    } else {
      // 如果没有错误，从 cache 中移除（如果之前有的话）
      if let Ok(mut cache) = self.cache.lock() {
        cache.remove(resource);
      }
    }
  }

  // let should_lint = self
  //   .cache
  //   .lock()
  //   .map(|mut cache| !cache.remove(resource))
  //   .unwrap_or(true);

  // if should_lint {
  //   eprintln!("succeed_module done");
  //   self.lint(resource).await?;
  // }

  Ok(())
}
