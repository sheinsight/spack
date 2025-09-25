use std::path::PathBuf;

use rspack_cacheable::cacheable;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ModuleHelper {
  output_dir: String,
}

impl ModuleHelper {
  pub fn new(output_dir: &str) -> Self {
    Self {
      output_dir: output_dir.to_string(),
    }
  }

  fn file_path_buf(&self, file_name: &str) -> PathBuf {
    PathBuf::from("@@").join(&self.output_dir).join(file_name)
  }

  pub fn file_name(&self, file_name: &str) -> String {
    self.file_path_buf(file_name).to_string_lossy().to_string()
  }

  pub fn file_name_with_bang(&self, file_name: &str) -> String {
    format!("!{}", self.file_name(file_name))
  }
}
