/// Asset 文件类型
#[derive(Debug, Clone, serde::Serialize)]
pub enum AssetType {
  JavaScript,
  CSS,
  Image,
  Font,
  Html,
  Json,
  Wasm,
  SourceMap,
  Other(String),
}

impl AssetType {
  /// 从文件名推断资源类型
  pub fn from_filename(filename: &str) -> Self {
    let extension = filename
      .rsplit('.')
      .next()
      .unwrap_or("")
      .to_lowercase();

    match extension.as_str() {
      // JavaScript
      "js" | "mjs" | "cjs" => Self::JavaScript,
      // CSS
      "css" => Self::CSS,
      // Images
      "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" | "avif" => Self::Image,
      // Fonts
      "woff" | "woff2" | "ttf" | "otf" | "eot" => Self::Font,
      // HTML
      "html" | "htm" => Self::Html,
      // JSON
      "json" => Self::Json,
      // WebAssembly
      "wasm" => Self::Wasm,
      // Source maps
      "map" => Self::SourceMap,
      // Other
      _ => Self::Other(extension),
    }
  }

  /// 转换为字符串表示
  pub fn as_str(&self) -> &str {
    match self {
      Self::JavaScript => "javascript",
      Self::CSS => "css",
      Self::Image => "image",
      Self::Font => "font",
      Self::Html => "html",
      Self::Json => "json",
      Self::Wasm => "wasm",
      Self::SourceMap => "sourcemap",
      Self::Other(ext) => ext.as_str(),
    }
  }
}

#[derive(Debug, serde::Serialize)]
pub struct Asset {
  // 文件名，如 "main.js"
  pub name: String,
  // 文件大小（原始大小）
  pub size: usize,
  // gzip 压缩后大小
  pub gzip_size: Option<usize>,
  // brotli 压缩后大小
  pub brotli_size: Option<usize>,
  // 关联的 chunk
  pub chunks: Vec<String>,
  // 是否实际输出
  pub emitted: bool,
  // 资源类型
  pub asset_type: AssetType,
}
