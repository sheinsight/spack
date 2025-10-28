use rspack_cacheable::cacheable;
use serde::Serialize;

#[cacheable]
#[derive(Debug, Clone, Serialize)]
pub struct Restricted {
  pub name: String,
  pub message: String,
}
