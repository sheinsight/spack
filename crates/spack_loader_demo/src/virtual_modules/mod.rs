use std::sync::{Arc, RwLock};

use rspack_fs::FileMetadata;
use rspack_paths::Utf8Path;

pub trait VirtualFileStore: Send + Sync {
  fn write_file(&mut self, path: &Utf8Path, content: Vec<u8>);

  fn get_file_content(&self, path: &Utf8Path) -> Option<&Vec<u8>>;

  fn get_file_metadata(&self, path: &Utf8Path) -> Option<FileMetadata>;

  fn read_dir(&self, path: &Utf8Path) -> Option<Vec<String>>;

  fn contains(&self, path: &Utf8Path) -> bool;
}

pub struct JsVirtualFileStore(Arc<RwLock<dyn VirtualFileStore>>);

impl std::fmt::Debug for dyn VirtualFileStore {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("VirtualFileStore").finish()
  }
}

#[derive(Debug)]
pub struct JsVirtualFile {
  pub path: String,
  pub content: String,
}

impl JsVirtualFileStore {
  pub fn new(store: Arc<RwLock<dyn VirtualFileStore>>) -> Self {
    Self(store)
  }

  pub fn write_virtual_file_sync(&self, path: String, content: String) {
    if let Ok(mut store) = self.0.write() {
      store.write_file(path.as_str().into(), content.into());
    }
  }

  pub fn batch_write_virtual_files_sync(&self, files: Vec<JsVirtualFile>) {
    if let Ok(mut store) = self.0.write() {
      for f in files {
        store.write_file(f.path.as_str().into(), f.content.into());
      }
    }
  }
}

mod fs;
pub use fs::VirtualFileSystem;

mod trie_store;
pub use trie_store::TrieVirtualFileStore;
