# 开发指南

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

## 项目特点

### 构建系统
- 使用 TypeScript 脚本 (`*.mts`) 进行构建和发布管理
- 支持多种构建 profile (dev/debug/release/profiling/ci)
- 自动化版本管理，从 rspack_core 版本同步

### 开发工具集成
- GitHub Actions 工作流用于 CI/CD
- 跨平台构建 (Cross Platform CI)
- 自动化发布流程 (Release workflow)