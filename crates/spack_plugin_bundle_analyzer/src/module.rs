use crate::module_type::ModuleType;

#[derive(Debug)]
pub struct Module {
  // 模块唯一 ID
  pub id: String,
  // 可读名称，如 "./src/index.js"
  pub name: String,
  // 模块大小
  pub size: u64,
  // 包含此模块的 chunks
  pub chunks: Vec<String>,
  // 模块类型
  pub module_type: ModuleType,
  // 是否来自 node_modules
  pub is_node_module: bool,
}
