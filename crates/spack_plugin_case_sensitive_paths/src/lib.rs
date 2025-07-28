#![feature(let_chains)]

use std::path::Path;

use derive_more::Debug;
use package_json_parser::PackageJsonParser;
use rspack_core::{
  ApplyContext, CompilationId, CompilerOptions, ModuleFactoryCreateData, NormalModuleCreateData,
  Plugin, PluginContext,
};
use rspack_error::Diagnostic;
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
mod opts;

pub use opts::CaseSensitivePathsPluginOpts;
use up_finder::{FindUpKind, UpFinder};

use crate::import_finder::ImportFinder;

#[plugin]
#[derive(Debug)]
pub struct CaseSensitivePathsPlugin {
  #[allow(unused)]
  options: CaseSensitivePathsPluginOpts,
}

impl CaseSensitivePathsPlugin {
  pub fn new(options: CaseSensitivePathsPluginOpts) -> Self {
    Self::new_inner(options)
  }

  // fn check_case_sensitive_path_optimized(
  //   &self,
  //   resource_path: &Path,
  //   raw_request: &str,
  //   current_file: &str,
  // ) -> Option<String> {
  //   if !resource_path.exists() {
  //     return None;
  //   }

  //   // 1. 首先检查完整路径的真实大小写
  //   let canonical_path = resource_path.canonicalize().ok()?;

  //   // 2. 比较请求的路径和真实路径
  //   if canonical_path.to_string_lossy() != resource_path.to_string_lossy() {
  //     let msg = format!(
  //       r#"Can't resolve {:?} in {:?}. (case mismatch)"#,
  //       raw_request, current_file
  //     );
  //     return Some(msg);
  //   }

  //   None
  // }

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
        let mut finder = ImportFinder::new(original_request.to_string());
        ast.visit(|program, _context| {
          program.visit_with(&mut finder);
        });
        finder.found_import
      }
      Err(_) => None,
    }
  }

  // 优化后的版本，参考 rspack 内部插件的做法：
  //   fn create_diagnostic_with_rspack(
  //     &self,
  //     error_message: &str,
  //     source_content: Option<&str>,
  //     import_position: Option<(usize, usize)>,
  //   ) -> Diagnostic {
  //     let title = "Module not found:".to_string();

  //     let error_message = format!("{error_message}");

  //     let help = r#"Fix the case of file paths to ensure consistency in cross-platform builds.
  // It may work fine on macOS/Windows, but will fail on Linux."#;

  //     let error = match (source_content, import_position) {
  //       (Some(source), Some((start, length))) => TraceableError::from_file(
  //         source.to_string(),
  //         start,
  //         start + length,
  //         title,
  //         error_message,
  //       )
  //       .with_help(Some(help))
  //       .with_hide_stack(Some(true))
  //       .boxed(),
  //       _ => TraceableError::from_lazy_file(0, 0, title, error_message)
  //         .with_help(Some(help))
  //         .with_hide_stack(Some(true))
  //         .boxed(),
  //     };

  //     Diagnostic::from(error)
  //   }
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

  fn clear_cache(&self, _id: CompilationId) {
    println!("clear_cache");
  }
}

#[plugin_hook(rspack_core::NormalModuleFactoryAfterResolve for CaseSensitivePathsPlugin)]
async fn after_resolve(
  &self,
  data: &mut ModuleFactoryCreateData,
  create_data: &mut NormalModuleCreateData,
) -> rspack_error::Result<Option<bool>> {
  let issuer = data.issuer.as_deref().unwrap_or("");

  if issuer.contains("node_modules") {
    return Ok(None);
  }

  let resource = &create_data.resource_resolve_data.resource;

  let resource_path = Path::new(resource);

  let Ok(source) = std::fs::read_to_string(issuer) else {
    return Ok(None);
  };

  let code = "case mismatch";

  let help = r#"Fix the case of file paths to ensure consistency in cross-platform builds.
It may work fine on macOS/Windows, but will fail on Linux."#;

  if resource.contains("node_modules")
    && vec!["./", "../", "/"]
      .into_iter()
      .all(|prefix: &'static str| !data.request.starts_with(prefix))
  {
    let finder = UpFinder::builder()
      .cwd(resource_path)
      .kind(FindUpKind::File)
      .build();

    let res = finder.find_up("package.json");

    if let Some(package_json) = res.first() {
      let package_json = PackageJsonParser::parse(package_json).unwrap();
      if let Some(name) = package_json.name {
        if !data.request.starts_with(&name.to_string()) {
          let error_message = format!(
            "Package name mismatch: request '{}' should start with package name '{}'",
            data.request,
            name.to_string()
          );

          if let Some(pos) = self.find_import_position(&source, &data.request, issuer) {
            let rewrite_label = miette::LabeledSpan::at(pos, "Path case mismatch");

            let diagnostic = miette::MietteDiagnostic::new(error_message)
              .with_code(code)
              .with_label(rewrite_label)
              .with_severity(miette::Severity::Error)
              .with_help(help);

            let named_source = miette::NamedSource::new(issuer, source.to_string());
            let report = miette::Report::new(diagnostic.to_owned()).with_source_code(named_source);
            let diagnostic = Diagnostic::from(report);
            data.diagnostics.push(diagnostic);
            return Ok(None);
          }
        }
      }
    }
  }

  let Ok(canonical_path) = resource_path.canonicalize() else {
    return Ok(None);
  };

  if canonical_path.to_string_lossy() == resource_path.to_string_lossy() {
    return Ok(None);
  }

  let error_message = format!(
    r#"Can't resolve {:?} in {:?}. (case mismatch)"#,
    &create_data.raw_request, issuer
  );

  let pos = self.find_import_position(&source, &create_data.raw_request, issuer);

  if let Some(pos) = pos {
    let rewrite_label = miette::LabeledSpan::at(pos, format!("Path case mismatch"));

    let diagnostic = miette::MietteDiagnostic::new(error_message)
      .with_code(code)
      .with_label(rewrite_label)
      .with_severity(miette::Severity::Error)
      .with_help(help);

    let named_source = miette::NamedSource::new(issuer, source.to_string());
    let report = miette::Report::new(diagnostic.to_owned()).with_source_code(named_source);
    let diagnostic = Diagnostic::from(report);
    data.diagnostics.push(diagnostic);
    return Ok(None);
  }

  Ok(None)
}
