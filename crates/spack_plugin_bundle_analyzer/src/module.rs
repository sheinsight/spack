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
  // 模块条件名称,用于模块解析条件判断(如 package.json 的 exports 字段)
  pub name_for_condition: String,
}
