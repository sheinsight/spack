use package_json_parser::PackageJsonParser;
use std::collections::HashMap;
use std::path::Path;
use up_finder::UpFinder;

/// 包版本解析器
///
/// 用于从 package.json 文件中解析 npm 包的版本号
/// 内置缓存机制，每个目录只会查找一次
pub struct PackageVersionResolver {
  // 缓存: 目录路径 -> 版本号
  cache: HashMap<String, String>,
}

impl PackageVersionResolver {
  /// 创建新的版本解析器
  pub fn new() -> Self {
    Self {
      cache: HashMap::new(),
    }
  }

  /// 解析包版本
  ///
  /// 参数:
  /// - module_path: 模块路径，如 "node_modules/react/index.js"
  ///
  /// 返回:
  /// - 版本号字符串，如果找不到返回 "unknown"
  pub fn resolve(&mut self, module_path: &str) -> String {
    // 1. 提取目录路径
    let dir = match Path::new(module_path).parent() {
      Some(d) => d,
      None => return "unknown".to_string(),
    };

    // 2. 使用目录路径作为缓存 key
    let cache_key = dir.to_string_lossy().to_string();

    // 3. 查缓存
    if let Some(version) = self.cache.get(&cache_key) {
      return version.clone();
    }

    // 4. 使用 up_finder 向上查找 package.json
    let version = self.find_package_version(dir);

    // 5. 写入缓存
    self.cache.insert(cache_key, version.clone());

    version
  }

  /// 向上查找最近的 package.json 并提取版本号
  fn find_package_version(&self, dir: &Path) -> String {
    // 使用 up_finder 从当前目录向上查找 package.json
    let up_finder = UpFinder::builder().cwd(dir).build();
    let paths = up_finder.find_up("package.json");

    // 遍历找到的 package.json 文件（从近到远）
    for path in paths.iter() {
      // 使用 package_json_parser 解析文件
      if let Ok(package_json) = PackageJsonParser::parse(path) {
        if let Some(version) = package_json.version {
          return version.to_string();
        }
      }
    }

    // 如果没找到或解析失败，返回 "unknown"
    "unknown".to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_resolver_cache() {
    let mut resolver = PackageVersionResolver::new();

    // 首次解析应该会查找文件
    let version1 = resolver.resolve("node_modules/react/index.js");

    // 同一目录的文件应该命中缓存
    let version2 = resolver.resolve("node_modules/react/cjs/react.development.js");

    // 两次结果应该相同
    assert_eq!(version1, version2);
  }
}
