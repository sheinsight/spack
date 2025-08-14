#![feature(let_chains)]

use std::{collections::HashSet, path::Path};

use derive_more::Debug;
use package_json_parser::PackageJsonParser;
use rspack_core::{
  ApplyContext, ModuleFactoryCreateData, NormalModuleCreateData, Plugin,
};
use rspack_error::Diagnostic;
use rspack_hook::{plugin, plugin_hook};
// 移除 swc_core 的直接导入以避免版本冲突

mod import_finder;
mod opts;

pub use opts::CaseSensitivePathsPluginOpts;
use up_finder::{FindUpKind, UpFinder};


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

  // 移除直接的语法解析，让 JavaScriptCompiler 自己处理

  // 简化版本：通过字符串匹配查找 import 位置
  fn find_import_position(
    &self,
    source_code: &str,
    original_request: &str,
    _file_path: impl Into<String>,
  ) -> Option<(usize, usize)> {
    // 简单的字符串匹配查找 import 语句
    let patterns = [
      format!("import '{}'", original_request),
      format!("import \"{}\"", original_request),
      format!("from '{}'", original_request),
      format!("from \"{}\"", original_request),
      format!("require('{}'", original_request),
      format!("require(\"{}\"", original_request),
    ];

    for pattern in &patterns {
      if let Some(start) = source_code.find(pattern) {
        return Some((start, start + pattern.len()));
      }
    }
    
    None
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

#[plugin_hook(rspack_core::NormalModuleFactoryAfterResolve for CaseSensitivePathsPlugin)]
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

  let resource_str = &create_data.resource_resolve_data.resource;

  let resource_path = Path::new(resource_str);

  let Ok(source) = std::fs::read_to_string(issuer) else {
    return Ok(None);
  };

  let code = "case mismatch";

  let help = r#"Fix the case of file paths to ensure consistency in cross-platform builds.
It may work fine on macOS/Windows, but will fail on Linux."#;

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
