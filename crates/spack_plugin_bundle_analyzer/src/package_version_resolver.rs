use std::collections::HashMap;
use std::path::Path;

use package_json_parser::PackageJsonParser;
use up_finder::UpFinder;

/// 包信息（包名、版本和 package.json 路径）
#[derive(Debug, Clone)]
pub struct PackageInfo {
  pub name: String,
  pub version: String,
  pub path: String,
}

/// 包信息解析器
///
/// 从 package.json 文件中解析 npm 包的名称和版本号
/// 内置缓存机制，每个目录只会查找一次
pub struct PackageVersionResolver {
  // 缓存: 目录路径 -> 包信息
  cache: HashMap<String, PackageInfo>,
}

impl PackageVersionResolver {
  /// 创建新的解析器
  pub fn new() -> Self {
    Self {
      cache: HashMap::new(),
    }
  }

  /// 解析包信息（包名、版本和 package.json 路径）
  ///
  /// 参数:
  /// - module_path: 模块路径，如 "node_modules/react/index.js"
  ///
  /// 返回:
  /// - Some(PackageInfo): 找到了 package.json
  /// - None: 不在 node_modules 中或找不到 package.json
  pub fn resolve(&mut self, module_path: &str) -> Option<PackageInfo> {
    // 1. 只处理 node_modules 中的模块
    if !module_path.contains("node_modules/") {
      return None;
    }

    // 2. 提取目录路径
    let dir = Path::new(module_path).parent()?;

    // 3. 使用目录路径作为缓存 key
    let cache_key = dir.to_string_lossy().to_string();

    // 4. 查缓存
    if let Some(info) = self.cache.get(&cache_key) {
      return Some(info.clone());
    }

    // 5. 使用 up_finder 向上查找 package.json
    let info = self.find_package_info(dir)?;

    // 6. 写入缓存
    self.cache.insert(cache_key, info.clone());

    Some(info)
  }

  /// 向上查找最近的 package.json 并提取包信息
  fn find_package_info(&self, dir: &Path) -> Option<PackageInfo> {
    // 使用 up_finder 从当前目录向上查找 package.json
    let up_finder = UpFinder::builder().cwd(dir).build();
    let paths = up_finder.find_up("package.json");

    // 遍历找到的 package.json 文件（从近到远）
    for path in paths.iter() {
      // 使用 package_json_parser 解析文件
      if let Ok(package_json) = PackageJsonParser::parse(path) {
        // 必须同时有 name 和 version 字段
        if let Some(name) = package_json.name {
          let version = package_json
            .version
            .map(|v| v.to_string())
            .unwrap_or_else(|| "unknown".to_string());

          return Some(PackageInfo {
            name: name.to_string(),
            version,
            path: path.to_string_lossy().to_string(),
          });
        }
      }
    }

    None
  }
}
