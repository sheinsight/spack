use swc_core::{
  common::{SourceMap, Spanned},
  ecma::{
    ast::{ImportDecl, Str},
    visit::{Visit, VisitWith},
  },
};

// AST Visitor 用于查找匹配的 import 语句
pub struct ImportFinder {
  pub target_patterns: Vec<String>,
  pub found_import: Option<(usize, usize, String)>, // (start, length, matched_pattern)
  pub source_map: std::sync::Arc<SourceMap>,
  pub debug: bool,
}

impl ImportFinder {
  pub fn new(
    target_patterns: Vec<String>,
    source_map: std::sync::Arc<SourceMap>,
    debug: bool,
  ) -> Self {
    Self {
      target_patterns,
      found_import: None,
      source_map,
      debug,
    }
  }

  fn path_matches(&self, import_source: &str) -> Option<String> {
    for pattern in &self.target_patterns {
      // 精确匹配
      if import_source == pattern {
        return Some(pattern.clone());
      }

      // 忽略大小写匹配
      if import_source.to_lowercase() == pattern.to_lowercase() {
        return Some(pattern.clone());
      }
    }
    None
  }
}

impl Visit for ImportFinder {
  fn visit_import_decl(&mut self, node: &ImportDecl) {
    if self.found_import.is_some() {
      return; // 已经找到了，不需要继续
    }

    let import_source = match &*node.src {
      Str { value, .. } => value.as_str(),
    };

    if self.debug {
      eprintln!("🔍 Found import: '{}'", import_source);
    }

    if let Some(matched_pattern) = self.path_matches(import_source) {
      // 使用字符串字面量的完整 span，并手动调整以包含引号
      let string_span = node.src.span();

      // SWC 的字符串字面量 span 可能不包含引号，我们需要手动调整
      let original_start = string_span.lo.0 as usize;
      let original_end = string_span.hi.0 as usize;

      // 手动调整：向前一个字符包含开引号，缩短长度以排除分号
      let adjusted_start = original_start.saturating_sub(1); // 包含开引号
                                                             // 计算字符串内容长度 + 2 个引号
      let content_length = import_source.len();
      let adjusted_length = content_length + 2; // 内容 + 两个引号

      if self.debug {
        let start_loc = self.source_map.lookup_char_pos(string_span.lo);
        let end_loc = self.source_map.lookup_char_pos(string_span.hi);
        eprintln!(
          "🔍 Original span: {:?} -> start: {}, end: {}",
          string_span, original_start, original_end
        );
        eprintln!(
          "🔍 Adjusted: start: {}, length: {} (content: {} + 2 quotes)",
          adjusted_start, adjusted_length, content_length
        );
        eprintln!(
          "🔍 Location: {}:{} - {}:{}",
          start_loc.line, start_loc.col_display, end_loc.line, end_loc.col_display
        );
      }

      self.found_import = Some((adjusted_start, adjusted_length, matched_pattern.clone()));

      if self.debug {
        eprintln!(
          "✅ Matched import '{}' with pattern '{}'",
          import_source, matched_pattern
        );
      }
    }

    // 继续访问子节点
    node.visit_children_with(self);
  }
}
