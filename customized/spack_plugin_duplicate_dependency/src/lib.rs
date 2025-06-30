#![feature(let_chains)]
use std::{collections::HashMap, path::PathBuf};

use derive_more::Debug;
use futures::future::BoxFuture;
use package_json_parser::PackageJsonParser;
use rspack_core::{
  ApplyContext, Compilation, CompilerFinishMake, CompilerOptions, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use up_finder::UpFinder;

pub type CompilationHookFn =
  Box<dyn Fn(Vec<Library>) -> BoxFuture<'static, Result<()>> + Sync + Send>;

#[derive(Debug)]
pub struct DuplicateDependencyPluginOptions {
  #[debug(skip)]
  pub on_detected: Option<CompilationHookFn>,
}

#[plugin]
#[derive(Debug)]
pub struct DuplicateDependencyPlugin {
  options: DuplicateDependencyPluginOptions,
}

impl DuplicateDependencyPlugin {
  pub fn new(options: DuplicateDependencyPluginOptions) -> Self {
    Self::new_inner(options)
  }
}

impl Plugin for DuplicateDependencyPlugin {
  fn name(&self) -> &'static str {
    "spack.DuplicateDependencyPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    // _ctx.context.compiler_hooks

    // _ctx
    //   .context
    //   .compilation_hooks
    //   .process_assets
    //   .tap(process_assets::new(self));

    ctx
      .context
      .compiler_hooks
      .finish_make
      .tap(finish_make::new(self));

    Ok(())
  }

  // fn clear_cache(&self, _id: CompilationId) {}

  // fn apply(
  //   &self,
  //   _ctx: PluginContext<&mut ApplyContext>,
  //   _options: &CompilerOptions,
  // ) -> Result<()> {
  //   Ok(())
  // }

  // fn clear_cache(&self, _id: CompilationId) {}
}

#[plugin_hook(CompilerFinishMake for DuplicateDependencyPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  let module_graph = compilation.get_module_graph();

  let mut cache = HashMap::new();

  for (_identifier, module) in module_graph.modules().iter() {
    if module.module_type().is_js_like()
      && let readable_name = module.readable_identifier(&compilation.options.context)
      && let Some(dir) = PathBuf::from(readable_name.to_string()).parent()
    {
      let up_finder = UpFinder::builder().cwd(dir).build();
      let paths = up_finder.find_up("package.json");

      if let Some(library) = paths
        .iter()
        .filter(|path| {
          let path_str = path.to_string_lossy().to_string();
          let cached = !cache.contains_key(path_str.as_str());
          let is_node_modules = path_str.contains("node_modules");
          cached && is_node_modules
        })
        .find_map(|path| {
          if let Ok(package_json) = PackageJsonParser::parse(path)
            && let Some(name) = package_json.name
            && let Some(version) = package_json.version
            && let Some(path) = package_json.__raw_path
          {
            return Some(Library::new(path.clone(), name.0, version.0));
          }
          return None;
        })
      {
        cache.insert(library.dir.clone(), library);
      }
    }
  }

  if let Some(on_detected) = &self.options.on_detected {
    let libraries = cache.values().cloned().collect::<Vec<_>>();
    on_detected(libraries).await?;
  }

  Ok(())
}

#[derive(Debug, Clone)]
pub struct Library {
  pub dir: String,
  pub name: String,
  pub version: String,
}

impl Library {
  pub fn new(dir: String, name: String, version: String) -> Self {
    Self { dir, name, version }
  }
}
