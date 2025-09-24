use std::path::PathBuf;

pub struct ModuleHelper {
  home_dir: String,
  output_dir: String,
}

impl ModuleHelper {
  pub fn new(home_dir: &str, output_dir: &str) -> Self {
    Self {
      home_dir: home_dir.to_string(),
      output_dir: output_dir.to_string(),
    }
  }

  pub fn file_path_buf(&self, file_name: &str) -> PathBuf {
    PathBuf::from(&self.home_dir)
      .join(&self.output_dir)
      .join(file_name)
  }

  pub fn file_path_buf_str(&self, file_name: &str) -> String {
    self.file_path_buf(file_name).to_string_lossy().to_string()
  }
}
