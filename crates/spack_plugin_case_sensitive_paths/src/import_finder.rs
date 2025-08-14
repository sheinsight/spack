// 简化的 import 查找器，避免 swc_core 版本冲突
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

// 简化版本：不使用实际的 AST 解析，因为版本冲突问题
// 这个结构体保留是为了保持接口兼容性，但实际逻辑已经移到主文件中