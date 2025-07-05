#![feature(let_chains)]

use std::{collections::HashMap, path::PathBuf, time::Instant};

use derive_more::Debug;
use package_json_parser::PackageJsonParser;
use rspack_core::{
  ApplyContext, Compilation, CompilerFinishMake, CompilerOptions, Plugin, PluginContext,
};
use rspack_hook::{plugin, plugin_hook};
use up_finder::UpFinder;
mod options;
mod response;

pub use options::{CompilationHookFn, DuplicateDependencyPluginOptions};
pub use response::{DuplicateDependencyPluginResponse, Library};

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

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &CompilerOptions,
  ) -> rspack_error::Result<()> {
    ctx
      .context
      .compiler_hooks
      .finish_make
      .tap(finish_make::new(self));
    Ok(())
  }
}

#[plugin_hook(CompilerFinishMake for DuplicateDependencyPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
  let start_time = Instant::now();

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

  let end_time = Instant::now();
  let duration = end_time.duration_since(start_time);

  let duration = duration.as_millis() as f64;

  let response =
    DuplicateDependencyPluginResponse::new(cache.values().cloned().collect(), duration);

  if let Some(on_detected) = &self.options.on_detected {
    on_detected(response).await?;
  }

  Ok(())
}

// #[allow(unused)]
// pub fn get_binding_plugin(_env: Env, options: Unknown<'_>) -> Result<BoxPlugin> {
//   let options = options.coerce_to_object()?;
//   // #[allow(clippy::disallowed_names, clippy::unwrap_used)]
//   let on_detected = options.get::<CompilationHookFn>("on_detected")?.unwrap();
//   // assert_eq!(foo, "bar".to_string());
//   Ok(Box::new(DuplicateDependencyPlugin::new(
//     DuplicateDependencyPluginOptions { on_detected: None },
//   )) as BoxPlugin)
// }

// register_plugin!("BindingBuilderTestingPlugin", get_binding_plugin);
