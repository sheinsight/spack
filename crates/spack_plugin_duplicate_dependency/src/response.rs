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

#[derive(Debug)]
pub struct DuplicateDependencyPluginResponse {
    pub libraries: Vec<Library>,
    pub duration: f64,
}

impl DuplicateDependencyPluginResponse {
    pub fn new(libraries: Vec<Library>, duration: f64) -> Self {
        Self {
            libraries,
            duration,
        }
    }
}
