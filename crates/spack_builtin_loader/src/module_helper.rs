use rspack_cacheable::cacheable;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ModuleHelper {
  import_prefix: String,
}

impl ModuleHelper {
  pub fn new(import_prefix: &str) -> Self {
    Self {
      import_prefix: import_prefix.to_string(),
    }
  }

  fn file_path(&self, file_name: &str) -> String {
    // 使用正斜杠拼接路径，避免 Windows 反斜杠在 JS 字符串中被当作转义字符
    format!("{}/{}", self.import_prefix, file_name)
  }

  pub fn file_name(&self, file_name: &str) -> String {
    self.file_path(file_name)
  }

  pub fn file_name_with_bang(&self, file_name: &str) -> String {
    format!("!{}", self.file_name(file_name))
  }
}
