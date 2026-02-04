# AGENTS.md

本文件为 Codex 提供仓库级指引。

## 项目概览
这是一个基于 Rust 与 Node.js 的 JS/TS 打包工具项目，底层构建引擎为 rspack。

## 目录提示
- Rust workspace 位于 `crates/*`
- Node 侧脚本与配置位于仓库根目录（`package.json`、`pnpm-workspace.yaml` 等）

## 工作约定
- 代码变更前先理解现有实现的设计思路
- 遵循项目现有的代码风格与架构模式
- 涉及 Rust 代码改动后，运行 `build:dev` 检查编译是否通过
- 默认使用中文沟通
