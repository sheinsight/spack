use crate::PackageVersion;

/// 重复的包
#[derive(Debug)]
pub struct DuplicatePackage {
  /// 包名
  pub name: String,
  /// 所有版本（按大小降序排序）
  pub versions: Vec<PackageVersion>,
  /// 总大小（所有版本加起来）
  pub total_size: u64,
  /// 浪费的空间（总大小 - 最大版本的大小）
  pub wasted_size: u64,
}
