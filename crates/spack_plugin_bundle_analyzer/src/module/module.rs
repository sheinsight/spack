use super::{ModuleKind, ModuleType};

/// 合并模块中的单个内部模块信息
#[derive(Debug, Clone, serde::Serialize)]
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

/// 模块依赖信息（出站依赖：当前模块依赖谁）
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModuleDependency {
  /// 被依赖模块的 ID
  pub module_id: String,
  /// 被依赖模块的名称
  pub module_name: String,
  /// 依赖类型（如 "esm-import", "cjs-require", "dynamic-import"）
  pub dependency_type: String,
}

/// 模块被引用原因（入站依赖：谁依赖了当前模块）
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModuleReason {
  /// 依赖当前模块的模块 ID
  pub module_id: String,
  /// 依赖当前模块的模块名称
  pub module_name: String,
  /// 依赖类型
  pub dependency_type: String,
}

#[derive(Debug, serde::Serialize)]
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
  /// 用户请求路径（如 require('lodash') 中的 'lodash'）
  pub user_request: Option<String>,
  /// 原始请求路径（如 loader 链中的完整请求）
  pub raw_request: Option<String>,
  /// 当前模块的出站依赖列表（当前模块依赖哪些模块）
  pub dependencies: Option<Vec<ModuleDependency>>,
  /// 当前模块的入站依赖列表（哪些模块依赖当前模块，为什么被包含）
  pub reasons: Option<Vec<ModuleReason>>,
}
