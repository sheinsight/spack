use std::io::Write as _;

use derive_more::{Deref, derive::Into};
use flate2::Compression;
use flate2::write::GzEncoder;
use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _};
use rspack_core::Compilation;

#[derive(Debug)]
pub struct Asset {
  // 文件名，如 "main.js"
  pub name: String,
  // 文件大小（原始大小）
  pub size: usize,
  // gzip 压缩后大小
  pub gzip_size: Option<usize>,
  // 关联的 chunk
  pub chunks: Vec<String>,
  // 是否实际输出
  pub emitted: bool,
}

#[derive(Debug, Default, Deref, Into)]
pub struct Assets(Vec<Asset>);

impl<'a> From<&'a mut Compilation> for Assets {
  fn from(compilation: &'a mut Compilation) -> Self {
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
        let gzip_size = if let Some(buffer) = buffer_opt {
          // 并行计算 gzip 压缩大小
          calculate_gzip_size(buffer)
        } else {
          None
        };

        Asset {
          name: name.clone(),
          size: *size,
          gzip_size,
          chunks: get_asset_chunks(name, compilation),
          emitted: true,
        }
      })
      .collect();
    Assets(assets)
  }
}

fn get_asset_chunks(asset_name: &str, compilation: &Compilation) -> Vec<String> {
  compilation
    .chunk_by_ukey
    .values()
    .filter(|chunk| chunk.files().contains(asset_name))
    .map(|chunk| {
      let id = if let Some(id) = chunk.id() {
        id.to_string()
      } else {
        "".to_string()
      };
      return id;
    })
    .collect()
}

/// 计算 gzip 压缩后的大小
///
/// 参数:
/// - data: 原始数据字节
///
/// 返回: 压缩后的字节数,如果压缩失败返回 None
fn calculate_gzip_size(data: &[u8]) -> Option<usize> {
  let mut encoder = GzEncoder::new(Vec::new(), Compression::default());

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
