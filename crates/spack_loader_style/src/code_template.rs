pub struct CodeTemplate {
  pub es_module: &'static str,
  pub cjs_module: &'static str,
}

impl CodeTemplate {
  pub fn new(es_module: &'static str, cjs_module: &'static str) -> Self {
    Self {
      es_module,
      cjs_module,
    }
  }

  pub fn code(&self, es_module: bool) -> String {
    if es_module {
      self.es_module.to_string()
    } else {
      self.cjs_module.to_string()
    }
  }
}
