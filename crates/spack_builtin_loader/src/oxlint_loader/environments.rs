use rspack_cacheable::cacheable;
use serde::Serialize;

#[cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct Environment {
  pub browser: bool,
  pub node: bool,
  pub commonjs: bool,
  pub es2024: bool,
  pub amd: bool,
  #[serde(rename = "shared-node-browser")]
  pub shared_node_browser: bool,
}

impl Default for Environment {
  fn default() -> Self {
    Self {
      node: true,
      browser: true,
      es2024: true,
      amd: false,
      commonjs: false,
      shared_node_browser: false,
    }
  }
}
