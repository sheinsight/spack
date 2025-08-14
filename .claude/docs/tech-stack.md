# 技术栈

## 核心依赖
- **Rust**: nightly-2025-05-30 版本
- **Node.js**: v22，使用 pnpm@10.11.0 包管理
- **rspack**: v0.4.10 (从 Cargo.toml 读取版本)
- **NAPI-RS**: v3.1.2 (用于 Rust/Node.js 绑定)
- **SWC**: v33.0.7 (JavaScript/TypeScript 编译)

## 测试框架
- **Vitest**: 主要测试框架
- 测试文件模式: `**/*.test.mts`
- 测试超时: 100 秒
- 快照测试: 使用自定义序列化器