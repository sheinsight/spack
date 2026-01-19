/// 单个版本的信息
#[derive(Debug)]
pub struct PackageVersion {
  pub version: String,
  pub size: u64,
  pub module_count: usize,
  pub package_json_path: String,
}
