use derive_more::derive::{Deref, Into};

use crate::{Module, Package, module::Modules, package::PackageBuilder};

#[derive(Debug, Deref, Into)]
pub struct Packages(pub Vec<Package>);

impl Packages {
  /// 从模块列表分析包信息，使用提供的 resolver（推荐）
  ///
  /// 参数:
  /// - modules: 模块列表
  /// - resolver: 可复用的包版本解析器（避免重复创建和缓存失效）
  pub fn from_with_resolver(
    modules: &Modules,
    resolver: &mut super::PackageVersionResolver,
  ) -> Self {
    let packages = analyze_packages(modules, resolver);
    Packages(packages)
  }
}

impl<'a> From<&'a Modules> for Packages {
  fn from(modules: &'a Modules) -> Self {
    let mut resolver = super::PackageVersionResolver::new();
    Self::from_with_resolver(modules, &mut resolver)
  }
}

/// 分析包依赖,按 package.json 路径聚合
///
/// 使用 package.json 路径作为唯一标识,能够准确区分:
/// - 同版本不同 peer 依赖的包实例 (pnpm 场景)
/// - 不同位置的相同包
///
/// 参数:
/// - modules: 模块列表
/// - resolver: 可复用的包版本解析器（避免重复创建和缓存失效）
fn analyze_packages(
  modules: &[Module],
  resolver: &mut super::PackageVersionResolver,
) -> Vec<Package> {
  use std::collections::HashMap;

  // key 是 package.json 路径, value 是包构建器
  let mut package_map: HashMap<String, PackageBuilder> = HashMap::new();

  // 1. 遍历所有模块,按 package.json 路径分组
  for module in modules {
    // 从 package.json 解析包名、版本和路径
    if let Some(info) = resolver.resolve(&module.name_for_condition) {
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
