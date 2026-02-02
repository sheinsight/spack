use std::fmt;

/// Module 的种类（rspack 内部类型）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleKind {
  /// 普通模块（最常见，约 92%）
  Normal,
  /// 合并模块（scope hoisting 优化产生，约 6%）
  Concatenated,
  /// 外部依赖（约 0.3%）
  External,
  /// 动态导入上下文（约 1.5%）
  Context,
  /// 内联代码模块（约 0.1%）
  Raw,
  /// 自引用模块（约 0.1%）
  SelfRef,
}

impl ModuleKind {
  /// 返回模块种类的字符串表示
  ///
  /// 注意：推荐使用 Display trait (`format!("{}", kind)`)
  pub fn as_str(&self) -> &'static str {
    match self {
      ModuleKind::Normal => "Normal",
      ModuleKind::Concatenated => "Concatenated",
      ModuleKind::External => "External",
      ModuleKind::Context => "Context",
      ModuleKind::Raw => "Raw",
      ModuleKind::SelfRef => "SelfRef",
    }
  }
}

impl fmt::Display for ModuleKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_str())
  }
}
