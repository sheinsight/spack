use std::{
  collections::HashMap,
  sync::{Arc, RwLock},
};

// 重新导出现有的 virtual_modules API
pub use rspack_binding_api::virtual_modules::{
  TrieVirtualFileStore, VirtualFileStore, VirtualFileSystem,
};
use rspack_core::{ApplyContext, Compilation, CompilationParams, CompilerCompilation, Plugin};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::Utf8PathBuf;

#[derive(Debug)]
pub struct VirtualModulesPluginOptions {
  pub modules: HashMap<String, String>,
}

impl VirtualModulesPluginOptions {
  pub fn new(modules: HashMap<String, String>) -> Self {
    Self { modules }
  }
}

#[plugin]
#[derive(Debug)]
pub struct VirtualModulesPlugin {
  options: VirtualModulesPluginOptions,
  virtual_file_store: Arc<RwLock<dyn VirtualFileStore>>,
}

impl VirtualModulesPlugin {
  pub fn new(options: VirtualModulesPluginOptions) -> Self {
    let store: Arc<RwLock<dyn VirtualFileStore>> =
      Arc::new(RwLock::new(TrieVirtualFileStore::new()));

    Self::new_inner(options, store)
  }

  pub fn with_store(
    options: VirtualModulesPluginOptions,
    store: Arc<RwLock<dyn VirtualFileStore>>,
  ) -> Self {
    Self::new_inner(options, store)
  }

  pub fn write_module(&self, path: &str, content: &str) -> Result<()> {
    if let Ok(mut store) = self.virtual_file_store.write() {
      // 确保路径是绝对路径，以避免 trie_store 中的下溢问题
      let absolute_path = if path.starts_with('/') {
        Utf8PathBuf::from(path)
      } else {
        Utf8PathBuf::from("/").join(path)
      };
      store.write_file(&absolute_path, content.as_bytes().to_vec());
    }
    Ok(())
  }

  pub fn get_virtual_file_store(&self) -> Arc<RwLock<dyn VirtualFileStore>> {
    self.virtual_file_store.clone()
  }
}

#[plugin_hook(CompilerCompilation for VirtualModulesPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let context = &compilation.options.context;

  for (file_path, content) in &self.options.modules {
    let full_path = if file_path.starts_with('/') {
      Utf8PathBuf::from(file_path)
    } else {
      // 确保路径以 / 开头以避免 trie_store 中的问题
      let relative_path = context.as_path().join(file_path);
      if relative_path.as_str().starts_with('/') {
        relative_path
      } else {
        Utf8PathBuf::from("/").join(relative_path)
      }
    };

    if let Ok(mut store) = self.virtual_file_store.write() {
      store.write_file(&full_path, content.as_bytes().to_vec());
    }
  }

  Ok(())
}

impl Plugin for VirtualModulesPlugin {
  fn name(&self) -> &'static str {
    "rspack.VirtualModulesPlugin"
  }

  fn apply(&self, ctx: &mut ApplyContext) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    Ok(())
  }
}
