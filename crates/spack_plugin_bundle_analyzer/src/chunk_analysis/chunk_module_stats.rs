/// Chunk 模块统计信息
#[derive(Debug, Clone)]
pub struct ChunkModuleStats {
  /// 模块总数
  pub total_modules: usize,
  /// 平均模块大小
  pub avg_module_size: u64,
  /// 中位数模块大小
  pub median_module_size: u64,
  /// 最大模块大小
  pub largest_module_size: u64,
  /// 最小模块大小
  pub smallest_module_size: u64,
}

impl ChunkModuleStats {
  /// 从模块列表计算统计信息
  pub fn from_sizes(sizes: &[u64]) -> Self {
    if sizes.is_empty() {
      return Self {
        total_modules: 0,
        avg_module_size: 0,
        median_module_size: 0,
        largest_module_size: 0,
        smallest_module_size: 0,
      };
    }

    let total_modules = sizes.len();
    let total_size: u64 = sizes.iter().sum();
    let avg_module_size = total_size / total_modules as u64;

    let mut sorted_sizes = sizes.to_vec();
    sorted_sizes.sort_unstable();

    let median_module_size = if total_modules % 2 == 0 {
      let mid = total_modules / 2;
      (sorted_sizes[mid - 1] + sorted_sizes[mid]) / 2
    } else {
      sorted_sizes[total_modules / 2]
    };

    let largest_module_size = *sorted_sizes.last().unwrap_or(&0);
    let smallest_module_size = *sorted_sizes.first().unwrap_or(&0);

    Self {
      total_modules,
      avg_module_size,
      median_module_size,
      largest_module_size,
      smallest_module_size,
    }
  }
}
