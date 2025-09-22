pub struct CodeTemplate {
  pub left: &'static str,
  pub right: &'static str,
}

impl CodeTemplate {
  pub fn new(left: &'static str, right: &'static str) -> Self {
    Self { left, right }
  }

  pub fn of_es_module(&self, es_module: bool) -> String {
    if es_module {
      self.left.to_string()
    } else {
      self.right.to_string()
    }
  }
}
