use derive_more::derive::{Deref, Into};

use crate::{Module, module::Modules, package_version_resolver::{self, PackageInfo}};

#[derive(Debug)]
pub struct Package {
  // 包名,如 "react" 或 "@babel/core"
  pub name: String,
  // 版本号(pnpm 可从路径提取,npm/yarn 为 "unknown")
  pub version: String,
  // 该包的总大小
  pub size: u64,
  // 包含的模块数量
  pub module_count: usize,
  // 该包包含的所有模块 ID 列表
  pub modules: Vec<String>,
  // package.json 文件路径
  pub package_json_path: String,
}

/// 包构建器（临时结构，仅用于 analyze_packages 函数内部）
struct PackageBuilder<'a> {
  info: PackageInfo,
  modules: Vec<&'a Module>,
}

impl<'a> PackageBuilder<'a> {
  fn new(info: PackageInfo) -> Self {
    Self {
      info,
      modules: Vec::new(),
    }
  }

  fn add_module(&mut self, module: &'a Module) {
    self.modules.push(module);
  }

  fn build(self) -> Package {
    let size: u64 = self.modules.iter().map(|m| m.size).sum();
    let modules: Vec<String> = self.modules.iter().map(|m| m.id.clone()).collect();

    Package {
      name: self.info.name,
      version: self.info.version,
      size,
      module_count: self.modules.len(),
      modules,
      package_json_path: self.info.path,
    }
  }
}

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
