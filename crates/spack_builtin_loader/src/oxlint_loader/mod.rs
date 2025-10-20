use std::sync::Arc;

use async_trait::async_trait;
use oxc::{
  allocator::Allocator,
  diagnostics::{GraphicalReportHandler, GraphicalTheme},
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
        "correctness": "off",
        "suspicious": "off",
        "pedantic": "off",
        "style": "off",
        "restriction": "off",
        "perf": "off",
        "nursery": "off"
      },
      "rules": {
        // eslint
        "eslint/constructor-super": [2],
        "eslint/for-direction":[2],
        "eslint/getter-return": [2, { "allowImplicit": true }],
        "eslint/no-async-promise-executor": [2],
        "eslint/no-case-declarations":[2],
        "eslint/no-class-assign": [2],
        "eslint/no-compare-neg-zero": [2],
        "eslint/no-cond-assign": [2,"except-parens"],
        "eslint/no-const-assign":[2],
        "eslint/no-constant-binary-expression":[2],
        "eslint/no-constant-condition":[2],
        "eslint/no-control-regex":[2],
        "eslint/no-delete-var":[2],
        "eslint/no-dupe-class-members":[2],
        "eslint/no-dupe-else-if":[2],
        "eslint/no-dupe-keys":[2],
        "eslint/no-duplicate-case":[2],
        "eslint/no-empty":[2,{"allowEmptyCatch":true}],
        "eslint/no-empty-character-class":[2],
        "eslint/no-empty-pattern": [2],
        "eslint/no-ex-assign":[2],
        // TODO 因为有 BUG 所以临时关闭
        "eslint/no-fallthrough":[0,{
            "allowEmptyCase":true
        }],
        "eslint/no-func-assign":[2],
        "eslint/no-global-assign":[2,{"exceptions":[]}],
        "eslint/no-import-assign":[2],
        // // 实际上只要禁用了 var 的使用，就只剩函数的场景会触发，因为只有 var、function 才会牵扯到提升问题
        "eslint/no-inner-declarations":[2,"functions"],
        "eslint/no-invalid-regexp":[2,{"allowConstructorFlags":[]}],
        "eslint/no-irregular-whitespace":[2,{}],
        "eslint/no-loss-of-precision":[2],
        "eslint/no-new-native-nonconstructor":[2],
        "eslint/no-nonoctal-decimal-escape":[2],
        "eslint/no-obj-calls":[2],
        "eslint/no-prototype-builtins":[2],
        "eslint/no-redeclare":[2,{ "builtinGlobals": false }],
        "eslint/no-regex-spaces":[2],
        "eslint/no-self-assign":[2],
        "eslint/no-setter-return":[2],
        "eslint/no-shadow-restricted-names":[2],
        "eslint/no-sparse-arrays":[2],
        "eslint/no-this-before-super":[2],
        "eslint/no-unexpected-multiline":[2],
        "eslint/no-unreachable":[2],
        "eslint/no-unsafe-finally":[2],
        "eslint/no-unsafe-negation":[2,{"enforceForOrderingRelations":true}],
        "eslint/no-unsafe-optional-chaining":[2],
        "eslint/no-unused-labels":[2],
        "eslint/no-useless-catch":[2],
        "eslint/no-useless-escape":[2],
        "eslint/use-isnan":[2,{"enforceForIndexOf": true}],
        "eslint/valid-typeof":[2],
        // jest
        // oxc
        // promise
        // react
        // typescript
        // unicorn
        "unicorn/new-for-builtins":[2],
        "unicorn/no-instanceof-array":[2],
        "unicorn/no-invalid-remove-event-listener":[2],
        "unicorn/no-thenable":[2],
        "unicorn/no-unreadable-array-destructuring":[2],
        "unicorn/require-array-join-separator":[2],
        "unicorn/require-number-to-fixed-digits-argument":[2]
      },
      "settings":{},
      "env":{},
      "globals":{},
      "overrides":[
        {
          "files": ["**/*.{ts,tsx,cts,mts}"],
          "env": {},
          "globals": {},
          "plugins": [],
          "rules":{
            "typescript/no-duplicate-enum-values":[2],
            "typescript/no-extra-non-null-assertion": [2],
            "typescript/no-misused-new": [2],
            "typescript/no-non-null-asserted-optional-chain": [2],
            "typescript/no-unsafe-function-type":[2],
            "typescript/no-unsafe-declaration-merging":[2],
            "typescript/no-wrapper-object-types":[2],
            "typescript/prefer-namespace-keyword":[2],
          }
        },
        {
          "files": ["*.{jsx,tsx}"],
          "env": {},
          "globals": {},
          "plugins": [],
          "rules":{
            "react/jsx-no-duplicate-props":[2],
            "react/jsx-no-target-blank":[2,{
              "enforceDynamicLinks": "always",
              "warnOnSpreadAttributes":false,
              "allowReferrer":false,
              "links":true,
              "forms":false
            }],
            "react/jsx-no-undef":[2],
            "react/no-children-prop":[2],
            "react/no-danger-with-children":[2],
            "react/no-direct-mutation-state":[2],
            "react/no-is-mounted":[2],
            "react/no-string-refs":[2],
            "react/jsx-no-comment-textnodes":[2],
            "react/no-render-return-value":[2],
            "react/no-find-dom-node":[2],
            "react/require-render-return": [2],
            "react/no-unescaped-entities":[2],
            // "react/react-in-jsx-scope": [match &self.react {
            //   Some(react) if react.runtime == crate::ReactRuntime::Automatic => 0,
            //   _ => 2
            // }],
          }
        }
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
    let handler = GraphicalReportHandler::new().with_theme(GraphicalTheme::unicode());

    // 收集所有 lint 错误信息
    let mut all_errors = Vec::new();

    // 将 lint 诊断信息推送到 rspack 的诊断系统
    for message in messages {
      let error = message.error;

      let mut output = String::new();
      handler
        .render_report(&mut output, &error)
        .map_err(|e| rspack_error::Error::from_error(e))?;

      // 打印带格式的错误信息
      println!("\n{}", output);

      all_errors.push(output);
    }

    // 如果有错误,返回错误以阻止编译
    if !all_errors.is_empty() {
      let combined_error = all_errors.join("\n\n");
      return Err(rspack_error::error!(format!(
        "Linting failed with {} error(s):\n\n{}",
        all_errors.len(),
        combined_error
      )));
    }

    loader_context.finish_with((source_code, sm));
    Ok(())
  }
}
