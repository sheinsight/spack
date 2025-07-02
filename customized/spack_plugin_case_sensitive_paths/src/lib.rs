use std::path::{Path, PathBuf};

use derive_more::Debug;
use rspack_core::{
  ApplyContext, CompilationId, CompilerOptions, ModuleFactoryCreateData, NormalModuleCreateData,
  Plugin, PluginContext,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};

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
                "File name case mismatch: requested '{}' but actual is '{}'\nPath: {}",
                file_name_str,
                entry_name,
                path.display()
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
      // 添加警告诊断到 data.diagnostics 中

      if let Ok(_source_content) = std::fs::read_to_string(current_file) {
        // let import_statements = JavaScriptCompiler::parse_import_statements(&source_content);
        let diagnostic = Diagnostic::error(
          "Path Case Sensitivity Issue".to_string(),
          format!("{}\n\nThis may cause inconsistent build results across different operating systems. Please fix the file path casing.", error_message)
        )
        .with_file(Some(current_file.to_string().into()));

        data.diagnostics.push(diagnostic);
      }

      if self.options.debug {
        // eprintln!("Warning: {}", error_message);
      }
    }
  }

  Ok(None)
}
