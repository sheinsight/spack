use derive_more::Debug;
use futures::future::BoxFuture;

use crate::resp::DuplicateDependencyPluginResp;

pub type CompilationHookFn = Box<
  dyn Fn(DuplicateDependencyPluginResp) -> BoxFuture<'static, Result<(), Box<dyn std::error::Error>>>
    + Sync
    + Send,
>;

// pub type CycleHandlerFn =
//   Box<dyn Fn(String, Vec<String>) -> BoxFuture<'static, Result<()>> + Sync + Send>;

//   pub type CompilationHookFn = Box<dyn Fn() -> BoxFuture<'static, Result<()>> + Sync + Send>;

#[derive(Debug)]
pub struct DuplicateDependencyPluginOpts {
  #[debug(skip)]
  pub on_detected: Option<CompilationHookFn>,
}
