# spack 项目概览

这是一个基于 Rust 和 Node.js 的 JavaScript/TypeScript 打包工具项目，使用 rspack 作为底层构建引擎。

## 技术栈

### 核心语言
- **Rust**: 主要后端逻辑，使用 nightly-2025-05-30 版本
- **TypeScript/JavaScript**: Node.js 绑定和构建脚本
- **Node.js**: 运行时环境，支持 Node.js 22

### 主要依赖和框架
- **rspack**: 核心打包引擎 (v0.4.8)
- **napi-rs**: Rust 到 Node.js 的绑定库 (v3.0.0-beta.9)
- **swc_core**: JavaScript/TypeScript 编译器 (v31.1.0)
- **pnpm**: 包管理工具 (v10.11.0)

### 项目结构
- **Workspace**: Cargo workspace 架构，包含多个 crates
- **多平台支持**: 支持 macOS、Windows、Linux (x86_64 和 aarch64)
- **WASM 支持**: 包含 WebAssembly 构建配置

## 插件系统
项目包含多个专用插件：
- `spack_plugin_bundle_analyzer`: 打包分析插件
- `spack_plugin_case_sensitive_paths`: 路径大小写敏感检查
- `spack_plugin_deadcode`: 死代码检测
- `spack_plugin_duplicate_dependency`: 重复依赖检测

## 构建和发布
- 使用 `build.mts` 脚本进行版本管理和发布
- 支持多种构建配置：dev、debug、release、profiling
- 自动化 Git 标签和版本发布流程

## 开发指南
- 使用 pnpm workspace 管理多包架构
- 支持增量编译和开发模式热重载
- 集成 Clippy 和 rustfmt 进行代码质量控制

---

- 请始终使用中文回答
