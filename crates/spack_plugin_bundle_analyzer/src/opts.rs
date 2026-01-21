use derive_more::Debug;
use futures::future::BoxFuture;

use crate::report::Report;

// use crate::resp::BundleAnalysisResult;

pub type CompilationHookFn =
  Box<dyn Fn(Report) -> BoxFuture<'static, Result<(), Box<dyn std::error::Error>>> + Sync + Send>;

#[derive(Debug)]
pub struct BundleAnalyzerPluginOpts {
  #[debug(skip)]
  pub on_analyzed: Option<CompilationHookFn>,
  /// 是否计算 gzip 压缩后的大小（默认：false）
  /// 注意：启用会增加构建时间
  pub gzip_assets: Option<bool>,
  /// 是否计算 brotli 压缩后的大小（默认：false）
  /// 注意：启用会增加构建时间，且比 gzip 慢 2-3 倍
  pub brotli_assets: Option<bool>,
}
