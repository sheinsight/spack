use super::{ModuleKind, ModuleType};

/// 合并模块中的单个内部模块信息
#[derive(Debug, Clone)]
pub struct ConcatenatedModuleInfo {
  /// 模块 ID
  pub id: String,
  /// 模块名称
  pub name: String,
  /// 模块大小
  pub size: u64,
  /// 模块文件类型（JavaScript/CSS/JSON 等）
  pub module_type: ModuleType,
  /// 是否来自 node_modules
  pub is_node_module: bool,
  /// 模块条件名称
  pub name_for_condition: String,
  /// 关联的 Package 的 package.json 路径（唯一标识）
  pub package_json_path: Option<String>,
}

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
  // 模块种类（rspack 内部类型：Normal/Concatenated/External 等）
  pub module_kind: ModuleKind,
  // 模块文件类型（JavaScript/CSS/JSON 等）
  pub module_type: ModuleType,
  // 是否来自 node_modules
  pub is_node_module: bool,
  // 模块条件名称,用于模块解析条件判断(如 package.json 的 exports 字段)
  pub name_for_condition: String,
  // 合并的模块列表（如果这是一个 ConcatenatedModule）
  pub concatenated_modules: Option<Vec<ConcatenatedModuleInfo>>,
  // 关联的 Package 的 package.json 路径（唯一标识）
  // 仅三方包模块有值，用于精确匹配对应的 Package
  pub package_json_path: Option<String>,
}
