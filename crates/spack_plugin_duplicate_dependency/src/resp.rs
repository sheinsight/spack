#[derive(Debug, Clone)]
pub struct Library {
  pub file: String,
  pub name: String,
  pub version: String,
}

impl Library {
  pub fn new(file: String, name: String, version: String) -> Self {
    Self {
      file,
      name,
      version,
    }
  }
}

#[derive(Debug, Clone)]
pub struct LibraryGroup {
  pub name: String,
  pub libs: Vec<Library>,
}

#[derive(Debug, Clone)]
pub struct DuplicateDependencyPluginResp {
  pub groups: Vec<LibraryGroup>,
  pub duration: f64,
}

impl DuplicateDependencyPluginResp {
  pub fn new(groups: Vec<LibraryGroup>, duration: f64) -> Self {
    Self { groups, duration }
  }
}
