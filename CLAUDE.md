# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# spack 项目概览

这是一个基于 Rust 和 Node.js 的 JavaScript/TypeScript 打包工具项目，使用 rspack 作为底层构建引擎。

## 核心架构

### Cargo Workspace 结构
项目使用 Rust workspace 管理多个 crates：
- `binding/`: 核心 NAPI 绑定模块，负责 Rust 到 Node.js 的桥接
- `spack_macros/`: 宏定义，包含插件注册和线程安全回调宏
- `spack_plugin_*`: 各种专用插件实现

### 插件系统架构
- 所有插件都通过 `binding/src/raws/` 模块暴露给 JavaScript
- 插件使用 `spack_macros::plugin_registry` 宏进行注册
- 每个插件实现自己的配置选项和业务逻辑

### 多平台构建
- 支持 8 个目标平台 (macOS/Windows/Linux 的 x64/arm64 架构)
- 每个平台生成独立的 npm 包 (`crates/binding/npm/*`)
- 使用 NAPI-RS 进行跨平台 Rust/Node.js 绑定

## 常用开发命令

### 构建命令
```bash
# 开发模式构建 (debug 配置)
pnpm run build:dev
# 等价于: pnpm --filter @shined/spack-binding run build:dev

# Rust binding 构建 (在 crates/binding/ 目录下)
pnpm run build          # release 模式
pnpm run build:debug    # debug 模式  
pnpm run build:ci       # CI 构建配置
pnpm run build:profiling # 性能分析构建
```

### 测试命令
```bash
# 运行所有测试
pnpm test
# 等价于: vitest run

# 测试文件位置: tests/*.test.mts
# 测试 fixtures: tests/fixtures/
```

### 发布命令
```bash
# 预发布 (更新版本和依赖)
node --experimental-strip-types release.mts prerelease

# 发布到 npm
node --experimental-strip-types release.mts publish
```

## 技术栈

### 核心依赖
- **Rust**: nightly-2025-05-30 版本
- **Node.js**: v22，使用 pnpm@10.11.0 包管理
- **rspack**: v0.4.10 (从 Cargo.toml 读取版本)
- **NAPI-RS**: v3.1.2 (用于 Rust/Node.js 绑定)
- **SWC**: v33.0.7 (JavaScript/TypeScript 编译)

### 测试框架
- **Vitest**: 主要测试框架
- 测试文件模式: `**/*.test.mts`
- 测试超时: 100 秒
- 快照测试: 使用自定义序列化器

## 插件开发

### 现有插件
- `spack_plugin_bundle_analyzer`: 打包分析插件
- `spack_plugin_case_sensitive_paths`: 路径大小写敏感检查
- `spack_plugin_deadcode`: 死代码检测  
- `spack_plugin_duplicate_dependency`: 重复依赖检测
- `spack_plugin_demo`: 示例插件

### 插件开发流程
1. 在 `crates/` 下创建新插件目录
2. 实现插件逻辑 (通常包含 `lib.rs`, `opts.rs`)
3. 在 `crates/binding/src/raws/` 添加对应的 raw 模块
4. 在 `crates/binding/src/lib.rs` 中注册插件
5. 在 workspace `Cargo.toml` 中添加依赖配置

## 项目特点

### 构建系统
- 使用 TypeScript 脚本 (`*.mts`) 进行构建和发布管理
- 支持多种构建 profile (dev/debug/release/profiling/ci)
- 自动化版本管理，从 rspack_core 版本同步

### 开发工具集成
- GitHub Actions 工作流用于 CI/CD
- 跨平台构建 (Cross Platform CI)
- 自动化发布流程 (Release workflow)

---

- 请始终使用中文回答