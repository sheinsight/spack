// 使用 AST 解析来准确查找 import 语句
use swc_core::{
  common::Spanned,
  ecma::{
    ast::{ImportDecl, Str},
    visit::{Visit, VisitWith},
  },
};

pub struct ImportFinder {
  pub target_request: String,
  pub found_import: Option<(usize, usize)>,
}

impl ImportFinder {
  pub fn new(target_request: String) -> Self {
    Self {
      target_request,
      found_import: None,
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

    // 检查是否匹配目标 import
    if import_source == self.target_request {
      // 获取字符串字面量的位置
      let span = node.src.span();
      let start = span.lo.0 as usize;
      let end = span.hi.0 as usize;

      self.found_import = Some((start, end - start));
    }

    node.visit_children_with(self);
  }
}
