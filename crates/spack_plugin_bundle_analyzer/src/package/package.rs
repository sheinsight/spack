#[derive(Debug, serde::Serialize)]
pub struct Package {
  // 包名,如 "react" 或 "@babel/core"
  pub name: String,
  // 版本号(pnpm 可从路径提取,npm/yarn 为 "unknown")
  pub version: String,
  // 该包的总大小
  pub size: u64,
  // 包含的模块数量
  pub module_count: usize,
  // 该包包含的所有模块数字 ID 列表（映射到原始路径，见 Report.module_id_map）
  pub modules: Vec<u32>,
  // package.json 文件路径
  pub package_json_path: String,
}
