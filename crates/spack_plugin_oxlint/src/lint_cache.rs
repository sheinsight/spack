use std::sync::{Arc, Mutex};

use oxc::diagnostics;
use oxc_linter::Message;
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Debug)]
pub struct LintCache {
  initialized: Arc<Mutex<bool>>,
  linted_files: Arc<Mutex<FxHashSet<String>>>,
  cache: Arc<Mutex<FxHashMap<String, Vec<Message>>>>,
}

impl LintCache {
  pub fn new() -> Self {
    Self {
      initialized: Arc::new(Mutex::new(false)),
      linted_files: Arc::new(Mutex::new(FxHashSet::default())),
      cache: Arc::new(Mutex::new(FxHashMap::default())),
    }
  }

  pub fn is_first_run(&self) -> bool {
    self
      .initialized
      .lock()
      .map(|mut initialized| {
        if !*initialized {
          *initialized = true;
          true
        } else {
          false
        }
      })
      .unwrap_or(false)
  }

  pub fn clear_linted_files(&self) {
    if let Ok(mut linted_files) = self.linted_files.lock() {
      linted_files.clear();
    }
  }

  pub fn mark_file_as_linted(&self, path: String) {
    if let Ok(mut linted_files) = self.linted_files.lock() {
      linted_files.insert(path);
    }
  }

  pub fn is_file_linted(&self, path: &str) -> bool {
    self
      .linted_files
      .lock()
      .map(|linted_files| linted_files.contains(path))
      .unwrap_or(false)
  }

  pub fn insert_cache(&self, path: String, messages: Vec<Message>) {
    if let Ok(mut cache) = self.cache.lock() {
      cache.insert(path, messages);
    }
  }

  pub fn remove_from_cache(&self, path: &str) {
    if let Ok(mut cache) = self.cache.lock() {
      cache.remove(path);
    }
  }

  pub fn get_error_count(&self) -> usize {
    self
      .cache
      .lock()
      .map(|c| {
        c.values()
          .flatten()
          .filter(|m| m.error.severity == diagnostics::Severity::Error)
          .count()
      })
      .unwrap_or(0)
  }

  pub fn mark_files_as_linted(&self, files: &[String]) {
    if let Ok(mut linted_files) = self.linted_files.lock() {
      for file in files {
        linted_files.insert(file.clone());
      }
    }
  }
}

