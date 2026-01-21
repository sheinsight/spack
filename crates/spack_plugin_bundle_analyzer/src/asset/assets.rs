use std::{collections::HashMap, io::Write as _};

use brotli::enc::BrotliEncoderParams;
use derive_more::derive::{Deref, Into};
use flate2::{Compression, write::GzEncoder};
use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _};
use rspack_core::Compilation;

use crate::{Asset, AssetType};

#[derive(Debug, Default, Deref, Into)]
pub struct Assets(Vec<Asset>);

impl<'a> From<&'a mut Compilation> for Assets {
  fn from(compilation: &'a mut Compilation) -> Self {
    Self::from_with_compression(compilation, false, false)
  }
}

impl Assets {
  /// 从 Compilation 中收集 Assets，可选择是否计算 gzip 和 brotli 大小
  pub fn from_with_compression(
    compilation: &mut Compilation,
    enable_gzip: bool,
    enable_brotli: bool,
  ) -> Self {
    // 预先构建 asset -> chunks 映射，避免对每个 asset 都遍历所有 chunks
    let asset_to_chunks = build_asset_chunks_map(compilation);

    let assets: Vec<_> = compilation
      .assets()
      .iter()
      .map(|(name, asset)| {
        let buffer = asset.source.as_ref().map(|s| s.buffer());
        let size = asset.source.as_ref().map(|s| s.size()).unwrap_or(0);
        (name.to_string(), size, buffer)
      })
      .collect();

    let assets = assets
      .par_iter()
      .map(|(name, size, buffer_opt)| {
        let gzip_size = if enable_gzip {
          buffer_opt.as_ref().and_then(|buffer| calculate_gzip_size(buffer))
        } else {
          None
        };

        let brotli_size = if enable_brotli {
          buffer_opt.as_ref().and_then(|buffer| calculate_brotli_size(buffer))
        } else {
          None
        };

        // 从预构建的映射中查找，O(1) 操作
        let chunks = asset_to_chunks
          .get(name.as_str())
          .cloned()
          .unwrap_or_default();

        Asset {
          name: name.clone(),
          size: *size,
          gzip_size,
          brotli_size,
          chunks,
          emitted: true,
          asset_type: AssetType::from_filename(name),
        }
      })
      .collect();
    Assets(assets)
  }
}

/// 构建 asset 名称到 chunk IDs 的映射
/// 一次遍历所有 chunks，避免对每个 asset 重复遍历
fn build_asset_chunks_map(compilation: &Compilation) -> HashMap<String, Vec<String>> {
  let mut map: HashMap<String, Vec<String>> = HashMap::new();

  for chunk in compilation.chunk_by_ukey.values() {
    let chunk_id = chunk.id().map(|id| id.to_string()).unwrap_or_default();

    // 将 chunk_id 添加到所有关联的 asset 中
    for file in chunk.files() {
      map
        .entry(file.to_string())
        .or_default()
        .push(chunk_id.clone());
    }
  }

  map
}

/// 计算 gzip 压缩后的大小
///
/// 参数:
/// - data: 原始数据字节
///
/// 返回: 压缩后的字节数,如果压缩失败返回 None
///
/// 注意: 使用快速压缩级别(1)以提升性能,因为我们只需要大小估算值
fn calculate_gzip_size(data: &[u8]) -> Option<usize> {
  // 使用压缩级别 1(最快),而非默认的级别 6
  // 对于大小估算来说,速度更重要,且大小差异在可接受范围内
  let mut encoder = GzEncoder::new(Vec::new(), Compression::new(1));

  // 写入数据
  if encoder.write_all(data).is_err() {
    return None;
  }

  // 完成压缩
  match encoder.finish() {
    Ok(compressed) => Some(compressed.len()),
    Err(e) => {
      tracing::error!("{}", e);
      None
    }
  }
}

/// 计算 brotli 压缩后的大小
///
/// 参数:
/// - data: 原始数据字节
///
/// 返回: 压缩后的字节数,如果压缩失败返回 None
///
/// 注意: 使用快速压缩质量(1)以提升性能,因为我们只需要大小估算值
fn calculate_brotli_size(data: &[u8]) -> Option<usize> {
  let mut output = Vec::new();

  // 使用压缩质量 1(最快),而非默认的质量 11
  // Brotli 的质量范围是 0-11,1 提供快速压缩但压缩率稍低
  let params = BrotliEncoderParams {
    quality: 1,
    ..Default::default()
  };

  match brotli::BrotliCompress(&mut &data[..], &mut output, &params) {
    Ok(_) => Some(output.len()),
    Err(e) => {
      tracing::error!("Brotli compression failed: {}", e);
      None
    }
  }
}
