#![feature(let_chains)]

use std::{path::Path, time::Instant};

use derive_more::Debug;
use itertools::Itertools as _;
use package_json_parser::PackageJsonParser;
use rspack_core::{
  ApplyContext, Compilation, CompilerAfterEmit, CompilerOptions, Plugin, PluginContext,
};
use rspack_hook::{plugin, plugin_hook};
use up_finder::UpFinder;
mod opts;
mod resp;
pub use opts::{CompilationHookFn, DuplicateDependencyPluginOpts};
pub use resp::{DuplicateDependencyPluginResp, Library, LibraryGroup};
use rustc_hash::{FxHashMap, FxHashSet};

#[plugin]
#[derive(Debug)]
pub struct DuplicateDependencyPlugin {
  options: DuplicateDependencyPluginOpts,
}

impl DuplicateDependencyPlugin {
  pub fn new(options: DuplicateDependencyPluginOpts) -> Self {
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
    options: &CompilerOptions,
  ) -> rspack_error::Result<()> {
    ctx
      .context
      .compiler_hooks
      .after_emit
      .tap(after_emit::new(self));

    Ok(())
  }
}

#[plugin_hook(CompilerAfterEmit for DuplicateDependencyPlugin)]
async fn after_emit(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
  let start_time = Instant::now();

  let module_graph = compilation.get_module_graph();

  let modules = module_graph.modules();

  let dir_iter = modules.iter().filter_map(|(_, module)| {
    let readable_name = module.readable_identifier(&compilation.options.context);
    if let Some(dir) = Path::new(readable_name.as_ref()).parent() {
      let res = dir.components().any(|c| c.as_os_str() == "node_modules");
      let is_js_file = module.module_type().is_js_like();
      if res && is_js_file {
        return Some(dir.to_path_buf());
      }
    }
    None
  });

  let mut cache = FxHashMap::default();
  let mut searched = FxHashSet::default();

  for dir in dir_iter {
    if !searched.insert(dir.clone()) {
      continue; // 已搜索过，跳过
    }

    let up_finder = UpFinder::builder().cwd(&dir).build();
    let paths = up_finder.find_up("package.json");
    let library = paths.iter().find_map(|p| {
      if let Ok(package_json) = PackageJsonParser::parse(p)
        && let Some(name) = package_json.name
        && let Some(version) = package_json.version
        && let Some(path) = package_json.__raw_path
      {
        return Some(Library::new(
          path.clone(),
          name.to_string(),
          version.to_string(),
        ));
      }
      None
    });

    if let Some(library) = library {
      cache.insert(dir.clone(), library);
    }
  }

  let duplicate_libraries: Vec<LibraryGroup> = cache
    .into_values()
    .into_group_map_by(|lib| (lib.name.clone(), lib.version.clone())) // 按name和version分组
    .into_iter()
    .into_group_map_by(|((name, _), _)| name.clone()) // 按name重新分组
    .into_iter()
    .filter(|(_, libs)| libs.len() > 1) // 过滤出有多个版本的包
    .map(|(name, groups)| LibraryGroup {
      name,
      libs: groups
        .into_iter()
        .map(|(_, libs)| libs[0].clone())
        .collect(),
    })
    .collect();

  let duration = start_time.elapsed().as_millis() as f64;

  let response = DuplicateDependencyPluginResp::new(duplicate_libraries, duration);

  if let Some(on_detected) = &self.options.on_detected {
    if let Err(e) = on_detected(response).await {
      println!("plugin-error: {:?}", e);
    }
  }

  Ok(())
}
