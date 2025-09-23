// #![feature(let_chains)]

use std::{collections::HashSet, path::Path};

use derive_more::Debug;
use package_json_parser::PackageJsonParser;
use rspack_core::{
  ApplyContext, Compilation, ModuleFactoryCreateData, NormalModuleCreateData, Plugin,
};
use rspack_error::Diagnostic;
use rspack_hook::{plugin, plugin_hook};
use rspack_javascript_compiler::JavaScriptCompiler;
// 使用与 rspack_javascript_compiler 相同版本的 swc_core
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

  // 使用 AST 解析查找 import 位置
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
}

impl Plugin for CaseSensitivePathsPlugin {
  fn name(&self) -> &'static str {
    "spack.CaseSensitivePathsPlugin"
  }

  fn apply(&self, ctx: &mut ApplyContext) -> rspack_error::Result<()> {
    ctx
      .normal_module_factory_hooks
      .after_resolve
      .tap(after_resolve::new(self));

    Ok(())
  }
}

#[plugin_hook(rspack_core::NormalModuleFactoryAfterResolve for CaseSensitivePathsPlugin,stage = Compilation::PROCESS_ASSETS_STAGE_ADDITIONAL)]
async fn after_resolve(
  &self,
  data: &mut ModuleFactoryCreateData,
  create_data: &mut NormalModuleCreateData,
) -> rspack_error::Result<Option<bool>> {
  let issuer = data.issuer.as_deref().unwrap_or("");

  // issuer 如果是 node_modules 的话, 说明这是三方包内部的逻辑, 不做任何验证
  if issuer.contains("node_modules") {
    return Ok(None);
  }

  let resource_str = create_data.resource_resolve_data.resource();

  let resource_path = Path::new(resource_str);

  let Ok(source) = std::fs::read_to_string(issuer) else {
    return Ok(None);
  };

  //   let code = "case mismatch";

  //   let help = r#"Fix the case of file paths to ensure consistency in cross-platform builds.
  // It may work fine on macOS/Windows, but will fail on Linux."#;

  // 如果 resource 包含 node_modules 的话, 说明是在引用三方包, 要考虑 npm alias 的情况
  // 但是如果是 / 开头的, 说明是绝对路径, 不做三方包相关的验证，直接跳到下面做路径匹配
  if resource_str.contains("node_modules/") && !create_data.raw_request.starts_with("/") {
    let file = data.options.context.as_path().join("package.json");

    // 解析不了 pkg , 这是一种异常 放过
    let Ok(package_json) = PackageJsonParser::parse(file) else {
      return Ok(None);
    };

    let mut dep_key_set: HashSet<String> = HashSet::new();

    if let Some(dependencies) = package_json.dependencies {
      for item in dependencies.keys() {
        dep_key_set.insert(item.to_string());
      }
    }

    if let Some(dev_dependencies) = package_json.dev_dependencies {
      for item in dev_dependencies.keys() {
        dep_key_set.insert(item.to_string());
      }
    }

    // 匹配 dependencies， 如果 request 是三方包的依赖, 放过， 主要考虑的是 别名的场景
    if dep_key_set.iter().any(|item| {
      create_data.raw_request.starts_with(&format!("{}/", item)) || create_data.raw_request == *item
    }) {
      return Ok(None);
    }

    let finder = UpFinder::builder()
      .cwd(resource_path)
      .kind(FindUpKind::File)
      .build();

    let res = finder.find_up("package.json");

    let Some(package_json) = res.first() else {
      return Ok(None);
    };

    let package_json = PackageJsonParser::parse(package_json).unwrap();
    let Some(name) = package_json.name else {
      return Ok(None);
    };

    if create_data.raw_request == name.to_string() {
      return Ok(None);
    }

    if create_data
      .raw_request
      .starts_with(&format!("{}/", name.to_string()))
    {
      return Ok(None);
    }

    let error_message = format!(
      r#"Can't resolve {:?} in {:?}. (case mismatch)"#,
      &create_data.raw_request, issuer
    );

    if let Some(pos) = self.find_import_position(&source, &data.request, issuer) {
      // let rewrite_label = miette::LabeledSpan::at(pos, "Path case mismatch");

      // let diagnostic = miette::MietteDiagnostic::new(error_message)
      //   .with_code(code)
      //   .with_label(rewrite_label)
      //   .with_severity(miette::Severity::Error)
      //   .with_help(help);

      let x = rspack_error::Error::from_string(
        Some(source.to_string()),
        pos.0,
        pos.1,
        "Path case mismatch".to_string(),
        error_message,
      );

      // let named_source = miette::NamedSource::new(issuer, source.to_string());
      // let report = miette::Report::new(diagnostic.to_owned()).with_source_code(named_source);

      let diagnostic = Diagnostic::from(x);
      data.diagnostics.push(diagnostic);
      return Ok(None);
    }
  }

  let Ok(canonical_path) = resource_path.canonicalize() else {
    return Ok(None);
  };

  if canonical_path.to_string_lossy() == resource_path.to_string_lossy() {
    return Ok(None);
  }

  let raw_request = create_data.raw_request.to_string();

  let error_message = format!(r#"Can't resolve {raw_request} in {issuer}. (case mismatch)"#);

  let pos = self.find_import_position(&source, &create_data.raw_request, issuer);

  if let Some(pos) = pos {
    // let rewrite_label = miette::LabeledSpan::at(pos, format!("Path case mismatch"));

    // let diagnostic = miette::MietteDiagnostic::new(error_message)
    //   .with_code(code)
    //   .with_label(rewrite_label)
    //   .with_severity(miette::Severity::Error)
    //   .with_help(help);

    // let named_source = miette::NamedSource::new(issuer, source.to_string());
    // let report = miette::Report::new(diagnostic.to_owned()).with_source_code(named_source);

    let error = rspack_error::Error::from_string(
      Some(source.to_string()),
      pos.0,
      pos.1,
      "Path case mismatch".to_string(),
      error_message,
    );

    let diagnostic = Diagnostic::from(error);
    data.diagnostics.push(diagnostic);
    return Ok(None);
  }

  Ok(None)
}
