use std::collections::HashMap;

use super::Summary;
use crate::{asset::Asset, chunk::Chunk, module::Module, package::Package};

// 核心数据结构
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
  // 元数据
  pub timestamp: u64,
  // 总览
  pub summary: Summary,
  // 数字 ID → 原始模块 ID 的映射表（用于减少 JSON 体积）
  // 注意：key 为字符串形式的数字，以兼容 NAPI
  pub module_id_map: HashMap<String, String>,
  // 资产（最终输出的文件）
  pub assets: Vec<Asset>,
  // 模块（源代码文件）
  pub modules: Vec<Module>,
  // 代码块
  pub chunks: Vec<Chunk>,
  // 包
  pub packages: Vec<Package>,
}
