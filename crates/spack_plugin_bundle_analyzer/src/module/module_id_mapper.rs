use std::collections::HashMap;

/// 模块 ID 映射器
///
/// 为每个模块分配一个唯一的数字 ID（从 0 开始的递增索引），
/// 用于减少 JSON 输出体积（将长路径字符串替换为短数字）
#[derive(Debug)]
pub struct ModuleIdMapper {
  /// 原始 ID → 数字 ID 的映射
  id_to_index: HashMap<String, u32>,
  /// 数字 ID → 原始 ID 的映射（用于最终输出）
  index_to_id: HashMap<u32, String>,
  /// 下一个可用的数字 ID
  next_index: u32,
}

impl ModuleIdMapper {
  /// 创建一个新的 ID 映射器
  pub fn new() -> Self {
    Self {
      id_to_index: HashMap::new(),
      index_to_id: HashMap::new(),
      next_index: 0,
    }
  }

  /// 获取或创建数字 ID
  ///
  /// 如果原始 ID 已存在，返回其对应的数字 ID；
  /// 否则分配一个新的数字 ID 并建立映射关系
  ///
  /// # 参数
  /// - `original_id`: 原始模块 ID（完整的文件路径）
  ///
  /// # 返回
  /// 对应的数字 ID（u32）
  pub fn get_or_create(&mut self, original_id: &str) -> u32 {
    // 如果已存在映射，直接返回
    if let Some(&index) = self.id_to_index.get(original_id) {
      return index;
    }

    // 分配新的数字 ID
    let index = self.next_index;
    self.next_index += 1;

    // 建立双向映射
    self.id_to_index.insert(original_id.to_string(), index);
    self.index_to_id.insert(index, original_id.to_string());

    index
  }

  /// 获取已存在的数字 ID（不创建新映射）
  ///
  /// # 参数
  /// - `original_id`: 原始模块 ID
  ///
  /// # 返回
  /// 如果原始 ID 已映射，返回 Some(数字 ID)；否则返回 None
  pub fn get(&self, original_id: &str) -> Option<u32> {
    self.id_to_index.get(original_id).copied()
  }

  /// 消费 mapper，返回数字 ID → 原始 ID 的映射表
  ///
  /// 该方法会移动 self 的所有权，返回最终的映射表用于序列化到 JSON
  /// 注意：为了兼容 NAPI，返回的 key 是字符串形式的数字
  ///
  /// # 返回
  /// HashMap<String, String> - 用于 Report.module_id_map（key 为数字的字符串表示）
  pub fn into_map(self) -> HashMap<String, String> {
    self
      .index_to_id
      .into_iter()
      .map(|(k, v)| (k.to_string(), v))
      .collect()
  }
}

impl Default for ModuleIdMapper {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_module_id_mapper() {
    let mut mapper = ModuleIdMapper::new();

    // 测试首次映射
    let id1 = mapper.get_or_create("./src/index.js");
    assert_eq!(id1, 0);

    // 测试重复映射（应返回相同 ID）
    let id1_again = mapper.get_or_create("./src/index.js");
    assert_eq!(id1_again, 0);

    // 测试第二个 ID
    let id2 = mapper.get_or_create("./src/utils.js");
    assert_eq!(id2, 1);

    // 测试 get 方法
    assert_eq!(mapper.get("./src/index.js"), Some(0));
    assert_eq!(mapper.get("./src/utils.js"), Some(1));
    assert_eq!(mapper.get("./src/unknown.js"), None);

    // 测试 into_map
    let map = mapper.into_map();
    assert_eq!(map.len(), 2);
    assert_eq!(map.get("0"), Some(&"./src/index.js".to_string()));
    assert_eq!(map.get("1"), Some(&"./src/utils.js".to_string()));
  }
}
