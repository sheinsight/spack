use std::path::Path;

/// 模块类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub enum ModuleType {
  JavaScript,
  TypeScript,
  CSS,
  Image,
  Font,
  JSON,
  WebAssembly,
  Unknown,
}

impl ModuleType {
  /// 根据文件路径判断模块类型
  pub fn from_path(path: &str) -> Self {
    let path = Path::new(path);

    // 获取文件扩展名
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    match extension.to_lowercase().as_str() {
      // JavaScript
      "js" | "jsx" | "mjs" | "cjs" => Self::JavaScript,

      // TypeScript
      "ts" | "tsx" | "mts" | "cts" => Self::TypeScript,

      // CSS
      "css" | "scss" | "sass" | "less" | "styl" | "stylus" => Self::CSS,

      // 图片
      "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" | "bmp" | "avif" => Self::Image,

      // 字体
      "woff" | "woff2" | "ttf" | "otf" | "eot" => Self::Font,

      // JSON
      "json" | "jsonc" => Self::JSON,

      // WebAssembly
      "wasm" => Self::WebAssembly,

      _ => Self::Unknown,
    }
  }

  /// 转换为字符串表示
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::JavaScript => "JavaScript",
      Self::TypeScript => "TypeScript",
      Self::CSS => "CSS",
      Self::Image => "Image",
      Self::Font => "Font",
      Self::JSON => "JSON",
      Self::WebAssembly => "WebAssembly",
      Self::Unknown => "Unknown",
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_javascript() {
    assert_eq!(ModuleType::from_path("index.js"), ModuleType::JavaScript);
    assert_eq!(ModuleType::from_path("app.jsx"), ModuleType::JavaScript);
    assert_eq!(ModuleType::from_path("module.mjs"), ModuleType::JavaScript);
    assert_eq!(ModuleType::from_path("module.cjs"), ModuleType::JavaScript);
  }

  #[test]
  fn test_typescript() {
    assert_eq!(ModuleType::from_path("index.ts"), ModuleType::TypeScript);
    assert_eq!(ModuleType::from_path("app.tsx"), ModuleType::TypeScript);
    assert_eq!(ModuleType::from_path("module.mts"), ModuleType::TypeScript);
  }

  #[test]
  fn test_css() {
    assert_eq!(ModuleType::from_path("style.css"), ModuleType::CSS);
    assert_eq!(ModuleType::from_path("style.scss"), ModuleType::CSS);
    assert_eq!(ModuleType::from_path("style.less"), ModuleType::CSS);
  }

  #[test]
  fn test_image() {
    assert_eq!(ModuleType::from_path("logo.png"), ModuleType::Image);
    assert_eq!(ModuleType::from_path("avatar.jpg"), ModuleType::Image);
    assert_eq!(ModuleType::from_path("icon.svg"), ModuleType::Image);
  }

  #[test]
  fn test_font() {
    assert_eq!(ModuleType::from_path("font.woff"), ModuleType::Font);
    assert_eq!(ModuleType::from_path("font.woff2"), ModuleType::Font);
    assert_eq!(ModuleType::from_path("font.ttf"), ModuleType::Font);
  }

  #[test]
  fn test_json() {
    assert_eq!(ModuleType::from_path("config.json"), ModuleType::JSON);
    assert_eq!(ModuleType::from_path("tsconfig.jsonc"), ModuleType::JSON);
  }

  #[test]
  fn test_wasm() {
    assert_eq!(
      ModuleType::from_path("module.wasm"),
      ModuleType::WebAssembly
    );
  }

  #[test]
  fn test_unknown() {
    assert_eq!(ModuleType::from_path("file.txt"), ModuleType::Unknown);
    assert_eq!(ModuleType::from_path("data.bin"), ModuleType::Unknown);
  }
}
