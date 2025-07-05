// 保留 import_finder.rs，但简化逻辑
use swc_core::{
  common::Spanned,
  ecma::{
    ast::{ImportDecl, Str},
    visit::{Visit, VisitWith},
  },
};

pub struct ImportFinder {
  pub target_request: String,               // 简化：只搜索一个目标
  pub found_import: Option<(usize, usize)>, // 移除 matched_pattern
  pub debug: bool,
}

impl ImportFinder {
  pub fn new(target_request: String, debug: bool) -> Self {
    Self {
      target_request,
      found_import: None,
      debug,
    }
  }
}

impl Visit for ImportFinder {
  fn visit_import_decl(&mut self, node: &ImportDecl) {
    if self.found_import.is_some() {
      return; // 已找到，不需要继续
    }

    let import_source = match &*node.src {
      Str { value, .. } => value.as_str(),
    };

    if self.debug {
      eprintln!("🔍 AST found import: '{}'", import_source);
    }

    // 简单的字符串匹配
    if import_source == self.target_request {
      // 获取字符串字面量的位置，包含引号
      let span = node.src.span();
      let start = span.lo.0 as usize - 1; // 包含开引号
      let length = import_source.len() + 2; // 内容 + 两个引号

      if self.debug {
        eprintln!("✅ AST matched: '{}' at position {}", import_source, start);
      }

      self.found_import = Some((start, length));
    }

    node.visit_children_with(self);
  }
}
