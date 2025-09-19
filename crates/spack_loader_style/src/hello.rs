use sailfish::TemplateSimple;

#[derive(TemplateSimple)]
#[template(path = "link_hmr_code.ejs")]
pub struct LinkHmrCodeTemplate {
  pub name: String,
  pub modulePath: String,
  pub esModule: bool,
}
