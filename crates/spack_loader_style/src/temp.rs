use sailfish::TemplateOnce;
use sailfish::TemplateSimple;

#[derive(TemplateSimple)]
#[template(path = "hello.stpl")]
pub struct LinkHmrCodeTemplate {
  pub name: String,
}
