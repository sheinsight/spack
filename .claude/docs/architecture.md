# 项目架构

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