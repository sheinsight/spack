#[derive(Debug, Clone)]
pub struct Library {
  pub dir: String,
  pub name: String,
  pub version: String,
}

impl Library {
  pub fn new(dir: String, name: String, version: String) -> Self {
    Self { dir, name, version }
  }
}

#[derive(Debug, Clone)]
pub struct LibraryGroup {
  pub name: String,
  pub libraries: Vec<Library>,
}

#[derive(Debug, Clone)]
pub struct DuplicateDependencyPluginResp {
  pub library_groups: Vec<LibraryGroup>,
  pub duration: f64,
}

impl DuplicateDependencyPluginResp {
  pub fn new(library_groups: Vec<LibraryGroup>, duration: f64) -> Self {
    Self {
      library_groups,
      duration,
    }
  }
}
