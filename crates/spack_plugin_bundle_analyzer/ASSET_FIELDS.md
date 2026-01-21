# Asset 结构字段扩展分析

本文档分析 Asset 数据结构的潜在字段扩展建议。

## 优先级说明

- **高优先级 (High)**: 显著提升分析能力，实现成本合理
- **中优先级 (Medium)**: 有价值但实现成本较高，或使用场景相对受限
- **低优先级 (Low)**: 价值较小或实现成本过高

---

## 当前 Asset 结构

```rust
pub struct Asset {
  pub name: String,
  pub size: usize,
  pub gzip_size: Option<usize>,
  pub chunks: Vec<String>,
  pub emitted: bool,
}
```

---

## 1. asset_type: String ⭐ 高优先级

### 实现复杂度

- **复杂度**: 低
- **实现方式**: 根据文件扩展名或 MIME 类型进行分类
- **代码量**: 约 30-50 行（添加类型判断逻辑）

```rust
pub enum AssetType {
  JavaScript,
  CSS,
  Image,
  Font,
  HTML,
  Other(String),
}
```

### 增加的数据量

- **每个 Asset**: 约 8-20 字节（字符串）
- **典型项目** (50 个 assets): 约 400 字节 - 1KB
- **大型项目** (500 个 assets): 约 4KB - 10KB
- **增长率**: 对总数据量影响 < 0.1%

### 性能开销

- **采集阶段**: 几乎为零（简单字符串匹配）
- **内存开销**: 可忽略（每个 asset 仅增加一个字段）
- **传输开销**: 可忽略（相比 asset 内容本身）

### 可实现功能列表

1. **资源类型统计**: 按类型分组显示资源数量和大小
2. **类型占比饼图**: 可视化不同类型资源的大小分布
3. **类型优化建议**:
   - 识别过大的图片资源
   - 检测未压缩的 CSS/JS
   - 发现意外的资源类型
4. **细粒度过滤**: 前端可按资源类型快速筛选
5. **性能分析**:
   - 分析不同类型资源的加载性能
   - 识别阻塞渲染的资源
6. **缓存策略建议**: 根据资源类型推荐缓存策略

---

## 2. brotli_size: Option<u32> 🔶 中优先级

### 实现复杂度

- **复杂度**: 中等
- **实现方式**: 集成 Brotli 压缩库（类似现有的 gzip）
- **代码量**: 约 50-100 行（参考 gzip 实现）
- **依赖**: 需添加 `brotli` crate
- **配置选项**: 需要添加可选配置开关

```rust
// 1. 在 opts.rs 中添加配置字段
pub struct BundleAnalyzerPluginOpts {
  pub on_analyzed: Option<CompilationHookFn>,
  pub gzip_assets: Option<bool>,
  /// 是否计算 brotli 压缩后的大小（默认：false）
  /// 注意：启用会增加构建时间，且比 gzip 慢 2-3 倍
  pub brotli_assets: Option<bool>,
}

// 2. 在 NAPI bindings (raw_bundle_analyzer.rs) 中同步
#[napi(object, object_to_js = false)]
pub struct RawBundleAnalyzerPluginOpts {
  pub on_analyzed: Option<ThreadsafeFunction<JsBundleAnalyzerPluginResp, ()>>,
  /// 是否计算 gzip 压缩后的大小（默认：false）
  pub gzip_assets: Option<bool>,
  /// 是否计算 brotli 压缩后的大小（默认：false）
  pub brotli_assets: Option<bool>,
}

// 3. 在 lib.rs 中使用配置
let enable_brotli = self.options.brotli_assets.unwrap_or(false);
let assets = Assets::from_with_compression(&mut *compilation, enable_gzip, enable_brotli);

// 4. 参考 gzip 实现 brotli 压缩逻辑
async fn calculate_brotli_size(content: &[u8]) -> Option<usize> {
  use brotli::enc::BrotliEncoderParams;
  // Brotli 压缩实现
}
```

### 增加的数据量

- **每个 Asset**: 4 字节（u32）
- **典型项目** (50 个 assets): 200 字节
- **大型项目** (500 个 assets): 2KB
- **增长率**: 对总数据量影响 < 0.05%

### 性能开销

- **采集阶段**: 中等
  - Brotli 压缩比 gzip 慢 2-3 倍
  - 但可以与 gzip 并行计算
  - 典型文件（100KB）: 约 50-150ms
- **内存开销**: 与 gzip 相当（压缩过程需要临时缓冲区）
- **配置要求**:
  - 必须通过 `brotli_assets: true` 显式启用
  - 默认值：`false`（不计算 brotli 大小）
  - 建议仅在需要精确传输大小评估时启用

### 可实现功能列表

1. **Brotli 压缩率分析**: 展示 Brotli 压缩后的总大小
2. **压缩算法对比**:
   - 原始 vs Gzip vs Brotli 三维对比
   - 压缩率差异可视化
3. **现代浏览器优化**:
   - 估算支持 Brotli 浏览器的实际传输大小
   - 计算 Brotli 带来的额外节省（通常比 gzip 小 15-20%）
4. **CDN 配置建议**:
   - 识别哪些 CDN 支持 Brotli
   - 提供 Brotli 配置指南
5. **ROI 分析**:
   - 计算启用 Brotli 的带宽节省
   - 评估压缩时间 vs 带宽节省的权衡

---

## 总结

### 建议实施顺序

1. **asset_type** (高优先级)
   - 实现简单，立即提升用户体验
   - 建议在第一批实现

2. **brotli_size** (中优先级)
   - 适合对传输大小敏感的项目
   - 需要通过配置选项 `brotli_assets: true` 显式启用
   - 默认值为 `false`，避免不必要的构建时间开销
   - 仅在需要精确评估现代浏览器传输大小时启用

### 性能对比

| 字段            | 采集开销          | 数据增长 | 传输影响 | 默认启用 |
| --------------- | ----------------- | -------- | -------- | -------- |
| asset_type      | ~0ms              | < 0.1%   | 低       | 待实现   |
| brotli_size     | 50-150ms/文件     | < 0.05%  | 低       | ❌ false |

### 配置示例

```typescript
// JavaScript/TypeScript 配置
import { registerBundleAnalyzerPlugin } from '@shined/spack-binding';

registerBundleAnalyzerPlugin();

const config = {
  plugins: [
    {
      name: 'BundleAnalyzerPlugin',
      options: {
        gzipAssets: true,     // 计算 gzip 大小
        brotliAssets: true,   // 计算 brotli 大小（默认 false）
        onAnalyzed: (report) => {
          console.log('Assets with compression info:', report.assets);
        }
      }
    }
  ]
};
```

```rust
// Rust 配置（内部实现）
let options = BundleAnalyzerPluginOpts {
  on_analyzed: Some(callback),
  gzip_assets: Some(true),
  brotli_assets: Some(true), // 默认 false，需显式启用
};
```
