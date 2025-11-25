use std::sync::Arc;

use rspack_collections::Identifier;
use rspack_error::Result;
use rspack_util::fx_hash::FxHashMap;
use tokio::sync::RwLock;

/// 通用的 loader 缓存
pub struct LoaderCache<T: Clone> {
  cache: RwLock<FxHashMap<(String, String), Arc<T>>>,
}

impl<T: Clone> LoaderCache<T> {
  pub fn new() -> Self {
    let cache = RwLock::new(FxHashMap::default());
    Self { cache }
  }

  pub async fn get_or_insert(
    &self,
    loader_request: &str,
    options: &str,
    create: impl FnOnce() -> Result<T>,
  ) -> Result<Arc<T>> {
    // 先查缓存
    let key = (loader_request.to_string(), options.to_string());
    if let Some(loader) = self.cache.read().await.get(&key) {
      return Ok(loader.clone());
    }

    // 创建新 loader
    let loader = Arc::new(create()?);

    // 缓存
    self.cache.write().await.insert(key, loader.clone());

    Ok(loader)
  }

  pub async fn clear(&self) {
    self.cache.write().await.clear();
  }
}

/// 通用的 Loader trait，支持自定义 identifier
pub trait LoaderWithIdentifier {
  fn with_identifier(self, identifier: Identifier) -> Self;
}
