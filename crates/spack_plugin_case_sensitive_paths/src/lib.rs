#![feature(let_chains)]

use std::path::Path;

use derive_more::Debug;
use napi::{Env, JsValue, Result, Unknown};
use rspack_core::{
  ApplyContext, BoxPlugin, CompilerOptions, ModuleFactoryCreateData, NormalModuleCreateData,
  Plugin, PluginContext,
};
use rspack_error::{Diagnostic, DiagnosticExt, TraceableError};
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

mod import_finder;
mod options;

use crate::{import_finder::ImportFinder, options::CaseSensitivePathsPluginOptions};

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

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &CompilerOptions,
  ) -> rspack_error::Result<()> {
    ctx
      .context
      .normal_module_factory_hooks
      .after_resolve
      .tap(after_resolve::new(self));

    Ok(())
  }
}

#[plugin_hook(rspack_core::NormalModuleFactoryAfterResolve for CaseSensitivePathsPlugin)]
async fn after_resolve(
  &self,
  data: &mut ModuleFactoryCreateData,
  create_data: &mut NormalModuleCreateData,
) -> rspack_error::Result<Option<bool>> {
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

pub fn get_binding_plugin(_env: Env, options: Unknown<'_>) -> Result<BoxPlugin> {
  let options = options.coerce_to_object()?;
  // #[allow(clippy::disallowed_names, clippy::unwrap_used)]
  // let foo = options.get::<CompilationHookFn>("on_detected")?.unwrap();
  // assert_eq!(foo, "bar".to_string());
  Ok(Box::new(CaseSensitivePathsPlugin::new(
    CaseSensitivePathsPluginOptions {
      debug: true,
      use_cache: true,
    },
  )) as BoxPlugin)
}
