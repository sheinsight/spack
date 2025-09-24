use std::path::PathBuf;

pub struct ModuleHelper {
  output_dir: String,
}

impl ModuleHelper {
  pub fn new(output_dir: &str) -> Self {
    Self {
      output_dir: output_dir.to_string(),
    }
  }

  pub fn file_path_buf(&self, file_name: &str) -> PathBuf {
    PathBuf::from("@@").join(&self.output_dir).join(file_name)
  }

  pub fn file_path_buf_str(&self, file_name: &str) -> String {
    self.file_path_buf(file_name).to_string_lossy().to_string()
  }
}
