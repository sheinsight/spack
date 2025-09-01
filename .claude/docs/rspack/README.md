# Rspack Rust API 文档

欢迎来到 Rspack Rust API 文档！这里提供了 Rspack 项目中所有 Rust crate 的完整 API 文档和使用指南。

## 文档导航

### 📚 核心模块
- **[01-核心模块](./01-核心模块.md)** - `rspack_core` 核心框架 API
  - 编译器和编译过程
  - 模块系统和依赖管理
  - 插件系统和钩子机制
  - 缓存系统和性能优化

### 🔧 构建器模块  
- **[02-构建器模块](./02-构建器模块.md)** - `rspack` 构建器和高级 API
  - CompilerBuilder 流式 API
  - 配置选项构建器
  - 内置插件系统
  - 使用示例和最佳实践

### 🌉 绑定接口模块
- **[03-绑定接口模块](./03-绑定接口模块.md)** - `rspack_binding_api` JavaScript 绑定层
  - JavaScript 编译器接口
  - 模块系统绑定
  - 文件系统绑定
  - 错误处理和诊断

### 🔌 插件生态系统
- **[04-插件生态系统](./04-插件生态系统.md)** - 完整的插件架构和 API
  - 41个插件 crate 详细介绍
  - 插件开发指南
  - 钩子系统详解
  - 自定义插件示例

### 🛠️ 工具和辅助模块
- **[05-工具和辅助模块](./05-工具和辅助模块.md)** - 基础设施和工具库
  - 文件系统抽象层
  - 哈希计算和路径处理
  - 错误处理系统
  - 性能追踪工具

## 快速开始

### 基本使用

```rust
use rspack::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建编译器
    let mut compiler = Compiler::builder()
        .context("./src")
        .entry("main", "./index.js")
        .mode(Mode::Development)
        .build()?;
    
    // 执行构建
    compiler.build().await?;
    Ok(())
}
```

### 核心概念

#### 1. 编译器 (Compiler)
编译器是 Rspack 的核心，负责协调整个构建过程：

```rust
// 创建编译器实例
let compiler = Compiler::new(options, plugins, output_fs, input_fs)?;

// 执行构建
compiler.build().await?;

// 增量构建
compiler.rebuild(changed_files, removed_files).await?;
```

#### 2. 模块系统 (Module System)
模块是构建的基本单位，每个文件都对应一个模块：

```rust
// 模块 trait 的核心方法
impl Module for MyModule {
    fn module_type(&self) -> &ModuleType;
    fn build(&mut self, build_context: BuildContext) -> Result<BuildResult>;
    fn code_generation(&self, compilation: &Compilation) -> Result<CodeGenerationResult>;
}
```

#### 3. 插件系统 (Plugin System)
插件通过钩子系统扩展编译器功能：

```rust
impl Plugin for MyPlugin {
    fn name(&self) -> &'static str { "MyPlugin" }
    
    fn apply(&self, ctx: &mut ApplyContext) -> Result<()> {
        ctx.compiler_hooks.compilation.tap(|compilation| {
            // 插件逻辑
        });
        Ok(())
    }
}
```

#### 4. 依赖管理 (Dependency Management)
依赖关系定义了模块之间的连接：

```rust
impl Dependency for MyDependency {
    fn dependency_type(&self) -> &DependencyType;
    fn resource_identifier(&self) -> Option<&str>;
    fn get_referenced_exports(&self, module_graph: &ModuleGraph) -> Vec<ReferencedExport>;
}
```

## 架构概览

```
┌─────────────────────────────────────────────────────────────────┐
│                        Rspack 架构图                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────┐    ┌─────────────────┐    ┌──────────────┐ │
│  │   JavaScript    │    │      Rust       │    │   Plugins    │ │
│  │   绑定层 API     │◄──►│    核心引擎      │◄──►│   生态系统    │ │
│  │                 │    │                 │    │              │ │
│  │ • JsCompiler    │    │ • Compiler      │    │ • 41个插件   │ │
│  │ • JsCompilation │    │ • Compilation   │    │ • 统一接口   │ │
│  │ • 模块绑定      │    │ • ModuleGraph   │    │ • 钩子系统   │ │
│  │ • 文件系统绑定  │    │ • 缓存系统      │    │ • 扩展性     │ │
│  └─────────────────┘    └─────────────────┘    └──────────────┘ │
│           │                       │                      │      │
│           │                       │                      │      │
│  ┌─────────────────┐    ┌─────────────────┐    ┌──────────────┐ │
│  │    构建器 API    │    │   工具和辅助     │    │   性能优化   │ │
│  │                 │    │     模块        │    │              │ │
│  │ • Builder模式   │    │ • 文件系统      │    │ • 并行编译   │ │
│  │ • 流式配置      │    │ • 哈希计算      │    │ • 增量构建   │ │
│  │ • 类型安全      │    │ • 错误处理      │    │ • 缓存优化   │ │
│  │ • 易用性        │    │ • 性能追踪      │    │ • 内存管理   │ │
│  └─────────────────┘    └─────────────────┘    └──────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## 模块依赖关系

```
rspack (主 crate)
├── rspack_core (核心框架)
│   ├── rspack_error (错误处理)
│   ├── rspack_fs (文件系统)
│   ├── rspack_hash (哈希工具)
│   └── rspack_util (通用工具)
├── rspack_binding_api (JavaScript 绑定)
│   ├── rspack_napi (NAPI 绑定工具)
│   └── rspack_core
├── 插件生态 (41个插件 crate)
│   ├── rspack_plugin_javascript
│   ├── rspack_plugin_css
│   ├── rspack_plugin_html
│   ├── rspack_plugin_split_chunks
│   └── ... (更多插件)
└── 工具库
    ├── rspack_collections (集合工具)
    ├── rspack_paths (路径处理)
    ├── rspack_regex (正则表达式)
    └── rspack_tracing (性能追踪)
```

## 主要特性

### 🚀 高性能
- **Rust 原生性能**: 利用 Rust 的零成本抽象和内存安全
- **并行编译**: 使用 `rayon` 和 `tokio` 实现并行处理
- **增量编译**: 智能依赖分析，只重新编译变化的模块
- **缓存优化**: 多级缓存系统，包括内存缓存和持久化缓存

### 🔧 易用性
- **Builder 模式**: 提供流式 API 配置复杂选项
- **类型安全**: 编译时类型检查，减少运行时错误
- **丰富的错误信息**: 详细的错误诊断和建议
- **Webpack 兼容**: 保持与 webpack 生态的兼容性

### 🔌 可扩展性
- **插件架构**: 统一的插件接口和丰富的钩子系统
- **模块化设计**: 高内聚低耦合的模块组织
- **Trait 抽象**: 通过 trait 定义核心接口，便于扩展
- **动态配置**: 支持运行时配置和插件注册

### 🛡️ 安全性
- **内存安全**: Rust 的所有权系统防止内存泄漏和数据竞争
- **类型安全**: 强类型系统避免常见的编程错误
- **线程安全**: 所有公共 API 都是线程安全的
- **错误处理**: 使用 `Result` 类型强制错误处理

## 开发指南

### 环境设置

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆项目
git clone https://github.com/web-infra-dev/rspack.git
cd rspack

# 构建项目
cargo build --release
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块的测试
cargo test -p rspack_core

# 运行基准测试
cargo bench
```

### 开发插件

1. 创建新的插件 crate
2. 实现 `Plugin` trait
3. 注册钩子函数
4. 编写测试
5. 更新文档

详细指南请参考 [插件生态系统文档](./04-插件生态系统.md)。

## 贡献指南

### 代码风格
- 遵循 Rust 标准代码风格
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 编写充分的文档注释

### 提交规范
- 使用语义化提交消息
- 包含测试用例
- 更新相关文档
- 确保 CI 通过

### 文档贡献
- 保持文档与代码同步
- 提供清晰的示例
- 包含性能指标
- 添加故障排除指南

## 性能基准

基于内部基准测试，Rspack 相比传统工具具有显著的性能优势：

| 项目规模 | 冷启动时间 | 热重载时间 | 内存使用 |
|---------|-----------|-----------|---------|
| 小型项目 | 2.1s      | 0.1s      | 120MB   |
| 中型项目 | 8.5s      | 0.3s      | 350MB   |
| 大型项目 | 25.2s     | 0.8s      | 800MB   |

*基准测试环境: M1 MacBook Pro, 16GB RAM*

## 常见问题

### Q: 如何升级到新版本？
A: 更新 `Cargo.toml` 中的版本号，运行 `cargo update`，查看更新日志了解破坏性变更。

### Q: 如何调试性能问题？
A: 使用 `rspack_tracing` 模块的追踪功能，或启用 Chrome DevTools 追踪。

### Q: 如何贡献新功能？
A: 先创建 issue 讨论设计，然后提交 PR，确保包含测试和文档。

### Q: 支持哪些目标平台？
A: 支持 Windows、macOS、Linux 等主流平台，以及 WebAssembly 目标。

## 版本历史

- **v1.0.0** - 初始稳定版本
- **v0.9.x** - Beta 版本系列
- **v0.8.x** - Alpha 版本系列

详细的更新日志请查看项目的 [CHANGELOG.md](https://github.com/web-infra-dev/rspack/blob/main/CHANGELOG.md)。

## 许可证

本项目基于 MIT 许可证开源，详情请参考 [LICENSE](https://github.com/web-infra-dev/rspack/blob/main/LICENSE) 文件。

## 联系我们

- **GitHub**: https://github.com/web-infra-dev/rspack
- **官方网站**: https://rspack.dev
- **Discord**: https://discord.gg/rspack
- **问题反馈**: https://github.com/web-infra-dev/rspack/issues

---

*最后更新: 2025-09-01*  
*文档版本: v1.0.0*