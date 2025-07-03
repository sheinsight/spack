use std::path::{Path, PathBuf};

use derive_more::Debug;
use rspack_core::{
  ApplyContext, CompilationId, CompilerOptions, ModuleFactoryCreateData, NormalModuleCreateData,
  Plugin, PluginContext,
};
use rspack_error::{
  miette::{LabeledSpan, MietteDiagnostic, Severity},
  Diagnostic, Result,
};
use rspack_hook::{plugin, plugin_hook};
use swc_core::{
  common::{FileName, SourceMap},
  ecma::{
    parser::{Lexer, Parser, StringInput, Syntax, TsSyntax},
    visit::VisitWith,
  },
};

use crate::import_finder::ImportFinder;

mod import_finder;

#[derive(Debug)]
pub struct CaseSensitivePathsPluginOptions {
  pub debug: bool,
  pub use_cache: bool,
}

#[plugin]
#[derive(Debug)]
pub struct CaseSensitivePathsPlugin {
  options: CaseSensitivePathsPluginOptions,
}

impl CaseSensitivePathsPlugin {
  pub fn new(options: CaseSensitivePathsPluginOptions) -> Self {
    Self::new_inner(options)
  }

  fn check_case_sensitive_path_optimized(&self, path: &Path) -> Option<String> {
    if !path.exists() {
      return None;
    }

    // 1. 首先检查完整路径的真实大小写
    let canonical_path = match path.canonicalize() {
      Ok(p) => p,
      Err(_) => return None,
    };

    // 2. 比较请求的路径和真实路径
    if canonical_path.to_string_lossy() != path.to_string_lossy() {
      return Some(format!(
        "Path case mismatch: requested '{}' but actual is '{}'",
        path.display(),
        canonical_path.display()
      ));
    }

    None
  }

  // 简化的 AST 匹配方法
  fn find_import_position(
    &self,
    source_code: &str,
    original_request: &str,
  ) -> Option<(usize, usize)> {
    // 创建 SourceMap
    let source_map = std::sync::Arc::new(SourceMap::default());
    let file = source_map.new_source_file(
      std::sync::Arc::new(FileName::Custom("temp.ts".to_string())),
      source_code.to_string(),
    );

    // 配置解析器语法
    let syntax = Syntax::Typescript(TsSyntax {
      tsx: true,
      decorators: true,
      dts: false,
      no_early_errors: false,
      disallow_ambiguous_jsx_like: false,
    });

    // 创建词法分析器和解析器
    let lexer = Lexer::new(syntax, Default::default(), StringInput::from(&*file), None);
    let mut parser = Parser::new_from(lexer);

    // 解析为模块
    match parser.parse_module() {
      Ok(module) => {
        let mut finder = ImportFinder::new(original_request.to_string(), self.options.debug);

        // 访问 AST
        module.visit_with(&mut finder);

        if let Some((start, length)) = finder.found_import {
          if self.options.debug {
            eprintln!(
              "✅ AST found import at byte offset {}, length {}",
              start, length
            );
          }
          return Some((start, length));
        }
      }
      Err(error) => {
        if self.options.debug {
          eprintln!("❌ Failed to parse file: {:?}", error);
        }
        // AST 解析失败时，回退到字符串匹配
        // return self.fallback_string_search(source_code, original_request);
        return None;
      }
    }

    None
  }
}

impl Plugin for CaseSensitivePathsPlugin {
  fn name(&self) -> &'static str {
    "spack.CaseSensitivePathsPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .normal_module_factory_hooks
      .after_resolve
      .tap(after_resolve::new(self));

    Ok(())
  }

  fn clear_cache(&self, _id: CompilationId) {}
}

#[plugin_hook(rspack_core::NormalModuleFactoryAfterResolve for CaseSensitivePathsPlugin)]
async fn after_resolve(
  &self,
  data: &mut ModuleFactoryCreateData,
  create_data: &mut NormalModuleCreateData,
) -> Result<Option<bool>> {
  let resource_path = &create_data.resource_resolve_data.resource;
  let current_file = data.issuer.as_deref().unwrap_or("");

  // 获取原始请求信息
  let dependency = data.dependencies[0]
    .as_module_dependency()
    .expect("should be module dependency");
  let original_request = dependency.request();
  let user_request = dependency.user_request();

  // 核心逻辑：检查路径大小写
  let path = Path::new(resource_path);

  if path.is_absolute() {
    if let Some(error_message) = self.check_case_sensitive_path_optimized(path) {
      // 使用 miette 创建友好的错误展示
      if let Ok(source_content) = std::fs::read_to_string(current_file) {
        let mut diagnostic = MietteDiagnostic::new(&error_message)
          .with_code("case-sensitive-paths")
          .with_severity(Severity::Error)
          .with_help(
            r#"Fix the case of file paths to ensure consistency in cross-platform builds. 

It may work fine on macOS/Windows, but will fail on Linux."#,
          );

        // 尝试找到 import 语句的位置并添加标签
        // 优先使用 original_request，如果找不到再尝试 user_request
        let search_request = if original_request != user_request {
          // 先试 user_request，因为它通常更接近源代码中的写法
          user_request
        } else {
          original_request
        };

        if let Some((start, length)) = self.find_import_position(&source_content, search_request) {
          diagnostic = diagnostic.with_label(LabeledSpan::new(
            Some("The path here has a case sensitivity issue.".to_string()),
            start,
            length,
          ));
        } else if search_request != original_request {
          // 如果 user_request 没找到，再试 original_request
          if let Some((start, length)) =
            self.find_import_position(&source_content, original_request)
          {
            diagnostic = diagnostic.with_label(LabeledSpan::new(
              Some("The path here has a case sensitivity issue.".to_string()),
              start,
              length,
            ));
          }
        }

        // 创建 miette::Error 并添加源代码
        let mut error = rspack_error::miette::Error::new(diagnostic);
        error = error.with_source_code(source_content.clone());

        // 转换为 rspack Diagnostic
        let diagnostic = Diagnostic::from(error);

        data.diagnostics.push(diagnostic);
      } else {
        let diagnostic = MietteDiagnostic::new(&error_message)
          .with_code("case-sensitive-paths")
          .with_severity(Severity::Error)
          .with_help(
            r#"Fix the case of file paths to ensure consistency in cross-platform builds. 

It may work fine on macOS/Windows, but will fail on Linux."#,
          );

        let error = rspack_error::miette::Error::new(diagnostic);
        let diagnostic = Diagnostic::from(error).with_file(Some(current_file.to_string().into()));

        data.diagnostics.push(diagnostic);
      }

      if self.options.debug {
        eprintln!("🔍 Case sensitivity warning: {}", &error_message);
      }
    }
  }

  Ok(None)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_simple_import_position_finding() {
    let options = CaseSensitivePathsPluginOptions {
      debug: false,
      use_cache: false,
    };
    let plugin = CaseSensitivePathsPlugin::new(options);

    let source_code = r#"
import React from 'react';
import { Component } from './MyComponent';
import utils from '../utils/helper';
import { Button } from './Button';
import styles from './styles.css';
"#;

    // 测试相对路径
    let result = plugin.find_import_position(source_code, "./MyComponent");
    assert!(result.is_some());
    if let Some((start, length)) = result {
      let found_text = &source_code[start..start + length];
      assert_eq!(found_text, "'./MyComponent'");
    }

    // 测试另一个相对路径
    let result = plugin.find_import_position(source_code, "./Button");
    assert!(result.is_some());
    if let Some((start, length)) = result {
      let found_text = &source_code[start..start + length];
      assert_eq!(found_text, "'./Button'");
    }

    // 测试不存在的路径
    let result = plugin.find_import_position(source_code, "./NonExistent");
    assert!(result.is_none());
  }

  #[test]
  fn test_different_quote_styles() {
    let options = CaseSensitivePathsPluginOptions {
      debug: true,
      use_cache: false,
    };
    let plugin = CaseSensitivePathsPlugin::new(options);

    let source_code = r#"
import React from "react";
import { Component } from './MyComponent';
import utils from '../utils/helper';
"#;

    // 测试双引号
    let result = plugin.find_import_position(source_code, "react");

    println!("result: {:?}", result);

    assert!(result.is_some());
    if let Some((start, length)) = result {
      let found_text = &source_code[start..start + length];
      assert_eq!(found_text, "\"react\"");
    }

    // 测试单引号
    let result = plugin.find_import_position(source_code, "./MyComponent");
    assert!(result.is_some());
    if let Some((start, length)) = result {
      let found_text = &source_code[start..start + length];
      assert_eq!(found_text, "'./MyComponent'");
    }

    // 测试模板字符串
    let result = plugin.find_import_position(source_code, "../utils/helper");
    assert!(result.is_some());
    if let Some((start, length)) = result {
      let found_text = &source_code[start..start + length];
      assert_eq!(found_text, "'../utils/helper'");
    }
  }
}
