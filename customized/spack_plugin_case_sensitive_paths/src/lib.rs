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
    parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax},
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

  // 核心方法：检查路径大小写
  fn check_case_sensitive_path(&self, path: &Path) -> Option<String> {
    if !path.exists() {
      return None;
    }

    // 简化逻辑：只检查文件名的大小写
    if let Some(parent) = path.parent() {
      if let Some(file_name) = path.file_name() {
        if let Ok(entries) = std::fs::read_dir(parent) {
          let file_name_str = file_name.to_string_lossy().to_string();

          for entry in entries.flatten() {
            let entry_name = entry.file_name().to_string_lossy().to_string();
            if entry_name.to_lowercase() == file_name_str.to_lowercase()
              && entry_name != file_name_str
            {
              return Some(format!(
                r#"
File name case mismatch: requested '{}' but actual is '{}'.

This may result in inconsistent build results on different operating systems.
"#,
                file_name_str, entry_name
              ));
            }
          }
        }
      }
    }

    // 检查路径中的每个组件
    let mut current_path = PathBuf::new();
    let components: Vec<_> = path.components().collect();

    for (i, component) in components.iter().enumerate() {
      current_path.push(component);

      if i == components.len() - 1 {
        // 最后一个组件（文件）已经在上面检查过了
        break;
      }

      if current_path.is_dir() {
        if let Some(parent) = current_path.parent() {
          if let Ok(entries) = std::fs::read_dir(parent) {
            let component_name = component.as_os_str().to_string_lossy().to_string();

            for entry in entries.flatten() {
              let entry_name = entry.file_name().to_string_lossy().to_string();
              if entry_name.to_lowercase() == component_name.to_lowercase()
                && entry_name != component_name
              {
                return Some(format!(
                  "Directory name case mismatch: requested '{}' but actual is '{}'\nPath: {}",
                  component_name,
                  entry_name,
                  current_path.display()
                ));
              }
            }
          }
        }
      }
    }

    None
  }

  // 生成可能的路径匹配模式
  fn generate_path_patterns(&self, resource_path: &str) -> Vec<String> {
    let path = Path::new(resource_path);
    let mut patterns = Vec::new();

    // 添加完整路径
    patterns.push(resource_path.to_string());

    if let Some(file_name) = path.file_name() {
      let file_name_str = file_name.to_string_lossy();
      patterns.push(file_name_str.to_string());

      if let Some(file_stem) = path.file_stem() {
        let file_stem_str = file_stem.to_string_lossy();
        patterns.push(file_stem_str.to_string());

        // 添加相对路径格式
        patterns.push(format!("./{}", file_name_str));
        patterns.push(format!("./{}", file_stem_str));
        patterns.push(format!("/{}", file_name_str));
        patterns.push(format!("/{}", file_stem_str));
      }
    }

    // 如果是相对路径，尝试提取更多可能的模式
    if let Some(parent) = path.parent() {
      let parent_str = parent.to_string_lossy();
      if !parent_str.is_empty() && parent_str != "." {
        let components: Vec<_> = parent.components().collect();
        if let Some(last_component) = components.last() {
          let last_dir = last_component.as_os_str().to_string_lossy();
          if let Some(file_name) = path.file_name() {
            let file_name_str = file_name.to_string_lossy();
            if let Some(file_stem) = path.file_stem() {
              let file_stem_str = file_stem.to_string_lossy();
              patterns.extend(vec![
                format!("./{}/{}", last_dir, file_name_str),
                format!("./{}/{}", last_dir, file_stem_str),
                format!("../{}/{}", last_dir, file_name_str),
                format!("../{}/{}", last_dir, file_stem_str),
              ]);
            }
          }
        }
      }
    }

    // 去重
    patterns.sort();
    patterns.dedup();

    if self.options.debug {
      eprintln!(
        "🔍 Generated patterns for '{}': {:?}",
        resource_path, patterns
      );
    }

    patterns
  }

  // 基于 AST 的精确 import 位置查找
  fn find_import_position(&self, source_code: &str, resource_path: &str) -> Option<(usize, usize)> {
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
      no_early_errors: true,
      disallow_ambiguous_jsx_like: true,
    });

    // 创建词法分析器和解析器
    let lexer = Lexer::new(syntax, Default::default(), StringInput::from(&*file), None);
    let mut parser = Parser::new_from(lexer);

    // 解析为模块
    match parser.parse_module() {
      Ok(module) => {
        let patterns = self.generate_path_patterns(resource_path);
        let mut finder = ImportFinder::new(patterns, source_map.clone(), self.options.debug);

        // 访问 AST
        module.visit_with(&mut finder);

        if let Some((start, length, matched_pattern)) = finder.found_import {
          if self.options.debug {
            eprintln!(
              "✅ AST found import at byte offset {}, length {}, pattern: '{}'",
              start, length, matched_pattern
            );
          }
          return Some((start, length));
        }
      }
      Err(error) => {
        if self.options.debug {
          eprintln!("❌ Failed to parse file: {:?}", error);
        }
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

  // 核心逻辑：检查路径大小写
  let path = Path::new(resource_path);

  if path.is_absolute() {
    if let Some(error_message) = self.check_case_sensitive_path(path) {
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
        if let Some((start, length)) = self.find_import_position(&source_content, resource_path) {
          diagnostic = diagnostic.with_label(LabeledSpan::new(
            Some("The path here has a case sensitivity issue.".to_string()),
            start,
            length,
          ));
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
  fn test_import_position_finding() {
    let options = CaseSensitivePathsPluginOptions {
      debug: false,
      use_cache: false,
    };
    let plugin = CaseSensitivePathsPlugin::new(options);

    let source_code = r#"
import React from 'react';
import { Component } from './MyComponent';
import utils from '../utils/helper';
import { Button } from './B';
import styles from './styles.css';
"#;

    // 测试相对路径
    let result = plugin.find_import_position(source_code, "./MyComponent");
    assert!(result.is_some());
    if let Some((start, length)) = result {
      let found_text = &source_code[start..start + length];
      assert_eq!(found_text, "'./MyComponent'");
      println!(
        "✅ AST found relative path: '{}' at position {}",
        found_text, start
      );
    }

    // 测试文件名匹配，模拟实际场景
    let full_path = "/Users/ityuany/GitRepository/rspack-demo/rspack-project/src/B.ts";
    let result = plugin.find_import_position(source_code, full_path);
    assert!(result.is_some());
    if let Some((start, length)) = result {
      let found_text = &source_code[start..start + length];
      assert_eq!(found_text, "'./B'");
      println!(
        "✅ AST found file import by name: '{}' for path '{}'",
        found_text, full_path
      );
    }

    // 测试不存在的路径
    let result = plugin.find_import_position(source_code, "./NonExistent");
    assert!(result.is_none());
    println!("✅ AST correctly handles non-existent imports");
  }

  #[test]
  fn test_ast_import_finder() {
    let source_code = r#"
import React from 'react';
import { useState, useEffect } from 'react';
import './App.css';
import { Button } from './components/Button';
import utils from '../utils/helper.ts';
"#;

    let options = CaseSensitivePathsPluginOptions {
      debug: false,
      use_cache: false,
    };
    let plugin = CaseSensitivePathsPlugin::new(options);

    // 测试多种路径格式
    let test_cases = vec![
      ("./App.css", "'./App.css'"),
      ("./components/Button", "'./components/Button'"),
      ("../utils/helper.ts", "'../utils/helper.ts'"),
      ("react", "'react'"),
    ];

    for (target_path, expected_text) in test_cases {
      let result = plugin.find_import_position(source_code, target_path);
      assert!(
        result.is_some(),
        "Failed to find import for '{}'",
        target_path
      );

      if let Some((start, length)) = result {
        let found_text = &source_code[start..start + length];
        assert_eq!(found_text, expected_text);
        println!(
          "✅ AST correctly found '{}' -> '{}'",
          target_path, found_text
        );
      }
    }
  }
}
