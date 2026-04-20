use derive_more::Debug;

#[derive(Debug)]
pub struct BundleAnalyzerPluginOpts {
  /// 报告输出目录（默认：当前工作目录）
  /// 支持相对路径和绝对路径
  pub output_dir: Option<String>,
  /// 是否计算 gzip 压缩后的大小（默认：false）
  /// 注意：启用会增加构建时间
  pub gzip_assets: Option<bool>,
  /// 是否计算 brotli 压缩后的大小（默认：false）
  /// 注意：启用会增加构建时间，且比 gzip 慢 2-3 倍
  pub brotli_assets: Option<bool>,
}
