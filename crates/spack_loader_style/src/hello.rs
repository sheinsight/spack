use sailfish::{TemplateOnce, TemplateSimple};

#[derive(TemplateSimple)]
#[template(path = "link_hmr_code.ejs")]
pub struct LinkHmrCodeTemplate {
  pub module_path: String,
  pub es_module: bool,
  pub insert_type: String,
  pub insert_module_path: String,
}

impl LinkHmrCodeTemplate {
  pub fn new(
    module_path: String,
    es_module: bool,
    insert_type: String,
    insert_module_path: String,
  ) -> Self {
    Self {
      module_path,
      es_module,
      insert_type,
      insert_module_path,
    }
  }
}
