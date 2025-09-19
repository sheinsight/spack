use sailfish::TemplateOnce;
use sailfish::TemplateSimple;

// use crate::template;

#[derive(TemplateSimple)]
#[template(path = "hello.stpl")]
pub struct LinkHmrCodeTemplate {
  pub name: String,
}
