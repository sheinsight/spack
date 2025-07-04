#![feature(let_chains)]
use std::path::Path;

use derive_more::Debug;
use rspack_core::{
  ApplyContext, CompilationId, CompilerOptions, ModuleFactoryCreateData, NormalModuleCreateData,
  Plugin, PluginContext,
};
use rspack_error::{Diagnostic, DiagnosticExt, Result, TraceableError};
use rspack_hook::{plugin, plugin_hook};
use rspack_javascript_compiler::JavaScriptCompiler;
use swc_core::{
  base::config::IsModule,
  common::FileName,
  ecma::{
    ast::EsVersion,
    parser::{EsSyntax, Syntax, TsSyntax},
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

  fn get_syntax_from_file_path(&self, file_path: impl Into<String>) -> Syntax {
    let file_path = file_path.into();
    let path = Path::new(&file_path);
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

    match extension {
      "ts" | "tsx" => Syntax::Typescript(TsSyntax {
        tsx: extension == "tsx",
        decorators: true,
        dts: false,
        no_early_errors: false,
        disallow_ambiguous_jsx_like: false,
        ..Default::default()
      }),
      "js" | "jsx" | "mjs" | "cjs" => Syntax::Es(EsSyntax {
        jsx: extension == "jsx",
        fn_bind: true,
        decorators: true,
        decorators_before_export: true,
        export_default_from: true,
        import_attributes: true,
        allow_super_outside_method: true,
        allow_return_outside_function: true,
        ..Default::default()
      }),
      _ => {
        // 默认使用 TypeScript 语法（兼容性最好）
        Syntax::Typescript(TsSyntax {
          tsx: true,
          decorators: true,
          dts: false,
          no_early_errors: false,
          disallow_ambiguous_jsx_like: false,
          ..Default::default()
        })
      }
    }
  }

  // 简化的 AST 匹配方法
  fn find_import_position(
    &self,
    source_code: &str,
    original_request: &str,
    file_path: impl Into<String>,
  ) -> Option<(usize, usize)> {
    let file_path = file_path.into();
    let syntax = self.get_syntax_from_file_path(&file_path);
    let filename = FileName::Custom(file_path);

    let compiler = JavaScriptCompiler::new();

    match compiler.parse(
      filename,
      source_code,
      EsVersion::EsNext,
      syntax,
      IsModule::Bool(true),
      None,
    ) {
      Ok(ast) => {
        let mut finder = ImportFinder::new(original_request.to_string(), self.options.debug);
        ast.visit(|program, _context| {
          program.visit_with(&mut finder);
        });
        finder.found_import
      }
      Err(_) => None,
    }
  }

  // 优化后的版本，参考 rspack 内部插件的做法：
  fn create_diagnostic_with_rspack(
    &self,
    error_message: &str,
    source_content: Option<&str>,
    import_position: Option<(usize, usize)>,
  ) -> Diagnostic {
    let title = "case-sensitive-paths".to_string();

    let error_message = format!("\n{error_message}\n");

    let help = r#"Fix the case of file paths to ensure consistency in cross-platform builds.
It may work fine on macOS/Windows, but will fail on Linux."#;

    let error = match (source_content, import_position) {
      (Some(source), Some((start, length))) => TraceableError::from_file(
        source.to_string(),
        start,
        start + length,
        title,
        error_message,
      )
      .with_help(Some(help))
      .with_hide_stack(Some(true))
      .boxed(),
      _ => TraceableError::from_lazy_file(0, 0, title, error_message)
        .with_help(Some(help))
        .with_hide_stack(Some(true))
        .boxed(),
    };

    Diagnostic::from(error)
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
  let resource_path = Path::new(resource_path);

  let current_file = data.issuer.as_deref().unwrap_or("");

  let first_dependency = data
    .dependencies
    .first()
    .and_then(|dep| dep.as_module_dependency());

  if let Some(dependency) = first_dependency
    && resource_path.is_absolute()
    && let Some(error_message) = self.check_case_sensitive_path_optimized(resource_path)
    && let Ok(source_content) = std::fs::read_to_string(current_file)
  {
    println!("data.issuer {:?}", data.issuer);
    let user_request = dependency.user_request();
    let diagnostic = self
      .find_import_position(&source_content, user_request, current_file)
      .map(|pos| {
        self.create_diagnostic_with_rspack(&error_message, Some(&source_content), Some(pos))
      })
      .unwrap_or_else(|| self.create_diagnostic_with_rspack(&error_message, None, None));

    data.diagnostics.push(diagnostic);
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
    let result = plugin.find_import_position(source_code, "./MyComponent", "test.ts");
    assert!(result.is_some());
    if let Some((start, length)) = result {
      let found_text = &source_code[start..start + length];
      assert_eq!(found_text, "'./MyComponent'");
    }

    // 测试另一个相对路径
    let result = plugin.find_import_position(source_code, "./Button", "test.ts");
    assert!(result.is_some());
    if let Some((start, length)) = result {
      let found_text = &source_code[start..start + length];
      assert_eq!(found_text, "'./Button'");
    }

    // 测试不存在的路径
    let result = plugin.find_import_position(source_code, "./NonExistent", "test.ts");
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
    let result = plugin.find_import_position(source_code, "react", "test.ts");

    println!("result: {:?}", result);

    assert!(result.is_some());
    if let Some((start, length)) = result {
      let found_text = &source_code[start..start + length];
      assert_eq!(found_text, "\"react\"");
    }

    // 测试单引号
    let result = plugin.find_import_position(source_code, "./MyComponent", "test.ts");
    assert!(result.is_some());
    if let Some((start, length)) = result {
      let found_text = &source_code[start..start + length];
      assert_eq!(found_text, "'./MyComponent'");
    }

    // 测试模板字符串
    let result = plugin.find_import_position(source_code, "../utils/helper", "test.ts");
    assert!(result.is_some());
    if let Some((start, length)) = result {
      let found_text = &source_code[start..start + length];
      assert_eq!(found_text, "'../utils/helper'");
    }
  }
}
