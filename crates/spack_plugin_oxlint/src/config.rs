use std::{collections::HashMap, path::Path};

use oxc_linter::Oxlintrc;
use serde_json::{Value, from_value, json, to_value};

use crate::{Environment, Restricted};

/// Oxlint 插件配置选项
#[derive(Debug, Clone)]
pub struct OxlintPluginOpts {
  pub output_dir: String,
  pub show_warning: bool,
  pub fail_on_error: bool,
  pub restricted_imports: Vec<Restricted>,
  pub restricted_globals: Vec<Restricted>,
  pub globals: HashMap<String, bool>,
  pub environments: Environment,
  pub config_file_path: Option<String>,
}

impl OxlintPluginOpts {
  /// 构建 Oxlintrc 配置
  ///
  /// 如果提供了 `oxlint_config_file_path`，则从文件加载；
  /// 否则根据选项生成配置并写入 `output_dir/.oxlintrc.json`
  pub fn build_oxlintrc(&self) -> Result<Oxlintrc, String> {
    if let Some(file_path) = &self.config_file_path {
      Self::load_from_file(file_path)
    } else {
      self.build_inner_oxlintrc()
    }
  }

  /// 从外部配置文件加载
  fn load_from_file(file_path: &str) -> Result<Oxlintrc, String> {
    Oxlintrc::from_file(Path::new(file_path)).map_err(|e| {
      format!(
        "Failed to load oxlintrc file: {}, error: {:?}",
        file_path, e
      )
    })
  }

  /// 从选项生成配置
  fn build_inner_oxlintrc(&self) -> Result<Oxlintrc, String> {
    // 1. 构建 JSON 配置
    let config_json = self
      .build_config_json()
      .map_err(|e| format!("Failed to build config JSON: {}", e))?;

    // 2. 写入配置文件（用于调试和审计）
    let config_output_path = Path::new(&self.output_dir).join(".oxlintrc.json");
    write_config_file(&config_json, &config_output_path, &self.output_dir)?;

    // 3. 反序列化为 Oxlintrc
    from_value::<Oxlintrc>(config_json)
      .map_err(|e| format!("Failed to deserialize Oxlintrc: {}", e))
  }

  /// 构建 Oxlint 配置的 JSON 表示
  ///
  /// 包含所有 lint 规则、环境、全局变量等配置
  fn build_config_json(&self) -> serde_json::Result<Value> {
    let restricted_imports = to_value(&self.restricted_imports)?;
    let restricted_globals = to_value(&self.restricted_globals)?;
    let globals = to_value(&self.globals)?;
    let environments = to_value(&self.environments)?;

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
        "no-restricted-globals": [1, restricted_globals],
        // TODO: 添加 restricted-imports 规则
        "no-restricted-imports": [1, {
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
}

/// 将配置写入文件
fn write_config_file(
  config: &serde_json::Value,
  output_path: &Path,
  output_dir: &str,
) -> Result<(), String> {
  // 格式化 JSON
  let pretty_config = serde_json::to_string_pretty(config)
    .map_err(|e| format!("Failed to pretty print config: {}", e))?;

  // 确保输出目录存在
  if !Path::new(output_dir).exists() {
    std::fs::create_dir_all(output_dir)
      .map_err(|e| format!("Failed to create output directory {}: {}", output_dir, e))?;
  }

  // 写入文件
  std::fs::write(output_path, pretty_config)
    .map_err(|e| format!("Failed to write config to {:?}: {}", output_path, e))?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_build_config_json() {
    let opts = OxlintPluginOpts {
      output_dir: "/tmp".to_string(),
      show_warning: true,
      fail_on_error: true,
      restricted_imports: vec![],
      restricted_globals: vec![],
      globals: HashMap::new(),
      environments: Environment::default(),
      config_file_path: None,
    };

    let result = opts.build_config_json();

    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.get("rules").is_some());
    assert!(config.get("plugins").is_some());
  }

  #[test]
  fn test_generate_oxlintrc() {
    use std::env;

    let temp_dir = env::temp_dir().join("test_oxlint_opts");
    let output_dir = temp_dir.to_string_lossy().to_string();

    let opts = OxlintPluginOpts {
      output_dir,
      show_warning: true,
      fail_on_error: true,
      restricted_imports: vec![],
      restricted_globals: vec![],
      globals: HashMap::new(),
      environments: Environment::default(),
      config_file_path: None,
    };

    let result = opts.build_oxlintrc();

    assert!(result.is_ok());

    // 清理
    let _ = std::fs::remove_dir_all(&temp_dir);
  }
}
