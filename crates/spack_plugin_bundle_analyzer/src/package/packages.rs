use derive_more::derive::{Deref, Into};

use crate::{Module, Package, module::Modules, package::PackageBuilder, package_version_resolver};

#[derive(Debug, Deref, Into)]
pub struct Packages(pub Vec<Package>);

impl<'a> From<&'a Modules> for Packages {
  fn from(modules: &'a Modules) -> Self {
    let packages = analyze_packages(&modules);
    Packages(packages)
  }
}

/// 分析包依赖,按 package.json 路径聚合
///
/// 使用 package.json 路径作为唯一标识,能够准确区分:
/// - 同版本不同 peer 依赖的包实例 (pnpm 场景)
/// - 不同位置的相同包
fn analyze_packages(modules: &[Module]) -> Vec<Package> {
  use std::collections::HashMap;

  // key 是 package.json 路径, value 是包构建器
  let mut package_map: HashMap<String, PackageBuilder> = HashMap::new();

  // 创建包信息解析器
  let mut resolver = package_version_resolver::PackageVersionResolver::new();

  // 1. 遍历所有模块,按 package.json 路径分组
  for module in modules {
    // 从 package.json 解析包名、版本和路径
    if let Some(info) = resolver.resolve(&module.name) {
      package_map
        .entry(info.path.clone())
        .or_insert_with(|| PackageBuilder::new(info))
        .add_module(module);
    }
  }

  // 2. 构建最终的 Package 列表
  let mut packages: Vec<Package> = package_map
    .into_values()
    .map(|builder| builder.build())
    .collect();

  // 3. 按大小降序排序
  packages.sort_by_key(|p| std::cmp::Reverse(p.size));

  packages
}
