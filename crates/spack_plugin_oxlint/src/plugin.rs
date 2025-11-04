use std::{path::Path, sync::Arc};

use ignore::WalkBuilder;
use rspack_core::{Compilation, CompilationParams, Plugin};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};

use crate::{lint_cache::LintCache, lint_runner::LintRunner, OxlintPluginOpts};

pub const OX_LINT_PLUGIN_IDENTIFIER: &'static str = "Spack.OxlintPlugin";

#[plugin]
#[derive(Debug)]
pub struct OxlintPlugin {
  #[allow(unused)]
  options: OxlintPluginOpts,
  lint_runner: Arc<LintRunner>,
  lint_cache: Arc<LintCache>,
}

impl OxlintPlugin {
  pub fn new(options: OxlintPluginOpts) -> Self {
    // 1. 构建配置
    let oxlintrc = options
      .build_oxlintrc()
      .expect("Failed to build oxlint config");

    let lint_cache = Arc::new(LintCache::new());

    let lint_runner = Arc::new(LintRunner::new(oxlintrc, options.show_warning));

    Self::new_inner(options, lint_runner, lint_cache)
  }
}

impl OxlintPlugin {
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
  let is_first_run = self.lint_cache.is_first_run();

  // 每次 this_compilation 开始时，清空 linted_files（标记当前编译周期）
  // 这样后续热更新时，succeed_module 中的文件可以正常 lint
  self.lint_cache.clear_linted_files();

  // 只在首次启动时执行全量 lint
  if !is_first_run {
    // 后续热更新时，只更新 diagnostics，不执行全量 lint
    let error_count = self.lint_cache.get_error_count();

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

  // 收集所有文件路径用于批量标记
  let file_paths: Vec<String> = files
    .iter()
    .map(|f| f.to_string_lossy().into_owned())
    .collect();

  // 记录所有 lint 过的文件（首次启动时，避免 succeed_module 重复 lint）
  self.lint_cache.mark_files_as_linted(&file_paths);

  for file in files {
    let resource = file.to_string_lossy().into_owned();

    let messages = self.lint_runner.lint(&file).await?;

    if !messages.is_empty() {
      self.lint_cache.insert_cache(resource, messages);
    }
  }

  let error_count = self.lint_cache.get_error_count();

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
  let should_lint = !self.lint_cache.is_file_linted(resource);

  if should_lint {
    // 执行 lint
    let messages = self.lint_runner.lint(resource).await?;

    // 标记为已 lint（避免同一个编译周期内重复 lint）
    self.lint_cache.mark_file_as_linted(resource.to_string());

    // 更新 cache
    if !messages.is_empty() {
      self.lint_cache.insert_cache(resource.to_string(), messages);
    } else {
      // 如果没有错误，从 cache 中移除（如果之前有的话）
      self.lint_cache.remove_from_cache(resource);
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
