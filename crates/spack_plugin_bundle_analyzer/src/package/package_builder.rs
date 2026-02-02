use crate::{Module, Package};

use super::version_resolver::PackageInfo;

/// 包构建器（临时结构，仅用于 analyze_packages 函数内部）
pub struct PackageBuilder<'a> {
  info: PackageInfo,
  modules: Vec<&'a Module>,
}

impl<'a> PackageBuilder<'a> {
  pub fn new(info: PackageInfo) -> Self {
    Self {
      info,
      modules: Vec::new(),
    }
  }

  pub fn add_module(&mut self, module: &'a Module) {
    self.modules.push(module);
  }

  pub fn build(self) -> Package {
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
