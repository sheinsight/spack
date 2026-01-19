use std::collections::HashMap;

use derive_more::derive::{Deref, Into};

use crate::{DuplicatePackage, Package, PackageVersion};

#[derive(Debug, Deref, Into)]
pub struct DuplicatePackages(pub Vec<DuplicatePackage>);

impl<'a> From<&'a [Package]> for DuplicatePackages {
  fn from(packages: &'a [Package]) -> Self {
    let duplicates = detect_duplicates(packages);
    DuplicatePackages(duplicates)
  }
}

/// 检测重复包
///
/// 定义：同一个包名有多个不同版本
fn detect_duplicates(packages: &[Package]) -> Vec<DuplicatePackage> {
  // 1. 按包名分组
  let mut grouped: HashMap<String, Vec<&Package>> = HashMap::new();

  for package in packages {
    grouped
      .entry(package.name.clone())
      .or_insert_with(Vec::new)
      .push(package);
  }

  // 2. 筛选出有多个版本的包（这才是重复）
  let mut duplicates: Vec<DuplicatePackage> = grouped
    .into_iter()
    .filter_map(|(name, pkg_list)| {
      // 只保留有多个版本的
      if pkg_list.len() <= 1 {
        return None;
      }

      // 转换为 PackageVersion
      let mut versions: Vec<PackageVersion> = pkg_list
        .iter()
        .map(|p| PackageVersion {
          version: p.version.clone(),
          size: p.size,
          module_count: p.module_count,
          package_json_path: p.package_json_path.clone(),
        })
        .collect();

      // 按大小降序排序（最大的版本在前面）
      versions.sort_by_key(|v| std::cmp::Reverse(v.size));

      // 计算总大小
      let total_size: u64 = versions.iter().map(|v| v.size).sum();

      // 浪费的空间 = 总大小 - 最大版本的大小
      // 假设：如果统一版本，只需要保留最大的那个版本
      let largest_size = versions[0].size;
      let wasted_size = total_size - largest_size;

      Some(DuplicatePackage {
        name,
        versions,
        total_size,
        wasted_size,
      })
    })
    .collect();

  // 3. 按浪费空间降序排序（最严重的放前面）
  duplicates.sort_by_key(|d| std::cmp::Reverse(d.wasted_size));

  duplicates
}
