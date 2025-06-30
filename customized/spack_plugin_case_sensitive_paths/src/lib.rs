use derive_more::Debug;
use rspack_core::{
  ApplyContext, CompilationId, CompilerOptions, ModuleFactoryCreateData, NormalModuleCreateData,
  Plugin, PluginContext,
};
use rspack_error::Result;
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
  _data: &mut ModuleFactoryCreateData,
  create_data: &mut NormalModuleCreateData,
) -> Result<Option<bool>> {
  // 最终解析的文件路径（通常是绝对路径）
  let resource_path = &create_data.resource_resolve_data.resource;

  // 用户在代码中写的原始路径
  let user_request = &create_data.user_request;

  // 处理后的路径
  let path = &create_data.request;

  println!("resource_path: {}", resource_path);
  println!("user_request: {}", user_request);
  println!("path: {}", path);

  // TODO: 在这里添加路径大小写检查逻辑
  // let path = std::path::Path::new(resource_path);
  // if path.is_absolute() && path.exists() {
  //   // 检查路径大小写
  // }

  Ok(None)
}
