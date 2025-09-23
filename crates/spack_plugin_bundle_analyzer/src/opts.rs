use derive_more::Debug;
use futures::future::BoxFuture;

use crate::resp::BundleAnalysisResult;

pub type CompilationHookFn = Box<
  dyn Fn(BundleAnalysisResult) -> BoxFuture<'static, Result<(), Box<dyn std::error::Error>>>
    + Sync
    + Send,
>;

#[derive(Debug)]
pub struct BundleAnalyzerPluginOpts {
  #[debug(skip)]
  pub on_analyzed: Option<CompilationHookFn>,
}
