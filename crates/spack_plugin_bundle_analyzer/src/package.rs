use derive_more::derive::{Deref, Into};

use crate::{Module, module::Modules, package_version_resolver};

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

#[derive(Debug, Deref, Into)]
pub struct Packages(pub Vec<Package>);

impl<'a> From<&'a Modules> for Packages {
  fn from(modules: &'a Modules) -> Self {
    let packages = analyze_packages(&modules);
    Packages(packages)
  }
}

/// 分析包依赖,按 (包名, 版本) 聚合
fn analyze_packages(modules: &[Module]) -> Vec<Package> {
  use std::collections::HashMap;

  // key 是 (包名, 版本) 元组, value 是 (package.json 路径, 模块列表)
  let mut package_map: HashMap<(String, String), (String, Vec<&Module>)> = HashMap::new();

  // 创建包信息解析器
  let mut resolver = package_version_resolver::PackageVersionResolver::new();

  // 1. 遍历所有模块,按 (包名, 版本) 分组
  for module in modules {
    // 从 package.json 解析包名、版本和路径
    if let Some(info) = resolver.resolve(&module.name) {
      package_map
        .entry((info.name, info.version))
        .or_insert_with(|| (info.path, Vec::new()))
        .1
        .push(module);
    }
  }

  // 2. 为每个包生成统计信息
  let mut packages: Vec<Package> = package_map
    .into_iter()
    .map(|((name, version), (package_json_path, mods))| {
      let size: u64 = mods.iter().map(|m| m.size).sum();
      let modules: Vec<String> = mods.iter().map(|m| m.id.clone()).collect();

      Package {
        name,
        version,
        size,
        module_count: mods.len(),
        modules,
        package_json_path,
      }
    })
    .collect();

  // 3. 按大小降序排序
  packages.sort_by_key(|p| std::cmp::Reverse(p.size));

  packages
}
