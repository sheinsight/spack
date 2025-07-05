use derive_more::Debug;
use futures::future::BoxFuture;

use crate::response::DuplicateDependencyPluginResponse;

pub type CompilationHookFn = Box<
  dyn Fn(DuplicateDependencyPluginResponse) -> BoxFuture<'static, rspack_error::Result<()>>
    + Sync
    + Send,
>;

#[derive(Debug)]
pub struct DuplicateDependencyPluginOptions {
  #[debug(skip)]
  pub on_detected: Option<CompilationHookFn>,
}
