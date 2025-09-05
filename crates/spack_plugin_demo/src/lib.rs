use std::sync::Arc;

use rspack_core::{
  ApplyContext, BoxLoader, Compilation, Context, ModuleRuleUseLoader,
  NormalModuleFactoryResolveLoader, Plugin, Resolver,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

// #[derive(Debug, Clone)]
// pub struct DemoPluginOpts {
//   // pub on_analyzed: Option<CompilationHookFn>,
// }

// #[plugin]
// #[derive(Debug)]
// pub struct DemoPlugin {
//   #[allow(unused)]
//   options: DemoPluginOpts,
// }

// impl DemoPlugin {
//   pub fn new(options: DemoPluginOpts) -> Self {
//     Self::new_inner(options)
//   }
// }

// impl Plugin for DemoPlugin {
//   fn name(&self) -> &'static str {
//     "spack.DemoPlugin"
//   }

//   fn apply(&self, ctx: &mut ApplyContext) -> rspack_error::Result<()> {
//     ctx
//       .compiler_hooks
//       .after_emit
//       .tap(after_emit::new(self));

//     Ok(())
//   }
// }

// #[plugin_hook(CompilerAfterEmit for DemoPlugin)]
// async fn after_emit(&self, compilation: &mut Compilation) -> rspack_error::Result<()> {
//   let start_time = Instant::now();

//   let stats = compilation.get_stats();

//   stats.get_chunks(
//     &ExtendedStatsOptions {
//       chunks: true,
//       chunk_modules: true,
//       assets: false,
//       cached_modules: false,
//       chunk_group_auxiliary: false,
//       chunk_group_children: false,
//       chunk_groups: false,
//       chunk_relations: false,
//       depth: false,
//       entrypoints: EntrypointsStatsOption::Bool(false),
//       errors: false,
//       hash: false,
//       ids: false,
//       modules: true,
//       module_assets: true,
//       nested_modules: true,
//       optimization_bailout: false,
//       provided_exports: false,
//       reasons: true,
//       source: false,
//       used_exports: false,
//       warnings: false,
//     },
//     |chunks| {
//       for chunk in chunks {
//         println!("=== Chunk 信息 ===");

//         // for file in chunk.files {
//         //   println!("file: {}", file);
//         // }

//         // for (key, value) in chunk.sizes {
//         //   println!("{}: {}", key, value);
//         // }

//         println!("id_hints: {:?}", chunk.id_hints);

//         // 基本信息
//         if let Some(id) = &chunk.id {
//           println!("Chunk ID: {}", id);
//         }
//         if !chunk.names.is_empty() {
//           println!("Chunk 名称: {:?}", chunk.names);
//         }
//         println!("Chunk 大小: {} bytes", chunk.size);

//         // 入口点信息
//         println!("是否为入口点: {}", chunk.entry);

//         // 包含的模块
//         if let Some(modules) = &chunk.modules {
//           println!("包含的模块数量: {}", modules.len());
//           // 添加这段来分析空 chunk
//           if modules.is_empty() {
//             println!("⚠️  空 chunk - 可能原因:");
//             println!("  - Runtime chunk: {}", chunk.entry);
//             println!("  - 文件列表: {:?}", chunk.files);
//             if let Some(reason) = &chunk.reason {
//               println!("  - 创建原因: {}", reason);
//             }
//           }

//           for module in modules {
//             if let Some(name) = &module.name {
//               // 过滤掉 node_modules 中的模块（可选）
//               if !name.contains("node_modules") {
//                 println!("  - 模块: {}", name);
//                 println!("    大小: {} bytes", module.size);
//               } else {
//                 // 显示 node_modules 中的大模块
//                 if module.size > 50000.0 {
//                   println!("  - 大型依赖: {}", name);
//                   println!("    大小: {} bytes", module.size);
//                 }
//               }

//               // 显示模块的导入原因
//               // if let Some(reasons) = &module.reasons {
//               //   for reason in reasons.iter() {
//               //     // 只显示前3个原因
//               //     if let Some(module_name) = &reason.module_name {
//               //       println!("    <- 被 {} 引用", module_name);
//               //     }
//               //   }
//               // }
//             } else {
//               // 处理无名模块 - 确保这段代码存在且完整
//               println!("  - 无名模块:");
//               println!("    原因: {:?}", module.reasons);
//               if let Some(identifier) = &module.identifier {
//                 println!("    标识符: {}", identifier);
//               }
//               println!("    大小: {} bytes", module.size);
//               // ... 其他调试信息
//             }
//           }
//         }

//         // 生成的文件
//         println!("生成的文件:");
//         for file in &chunk.files {
//           println!("  - {}", file);
//         }

//         println!(""); // 空行分隔
//       }
//     },
//   )?;

//   println!(
//     "DemoPlugin 执行耗时: {:?} ms",
//     start_time.elapsed().as_millis()
//   );

//   Ok(())
// }

#[derive(Debug)]
pub struct JsLoaderRspackPluginOpts {}

#[plugin]
#[derive(Debug)]
pub struct JsLoaderRspackPlugin {
  #[allow(unused)]
  options: JsLoaderRspackPluginOpts,
  // compiler_id: once_cell::sync::OnceCell<CompilerId>,
  // pub(crate) runner_getter: JsLoaderRunnerGetter,
  // /// This complex data structure is used to avoid deadlock when running loaders which contain `importModule`
  // /// See: https://github.com/web-infra-dev/rspack/pull/10632
  // pub(crate) runner: Mutex<Arc<tokio::sync::OnceCell<JsLoaderRunner>>>,
  // pub(crate) loaders_without_pitch: RwLock<FxHashSet<String>>,
}

impl JsLoaderRspackPlugin {
  pub fn new(options: JsLoaderRspackPluginOpts) -> Self {
    Self::new_inner(options)
  }
}

impl Plugin for JsLoaderRspackPlugin {
  fn name(&self) -> &'static str {
    "spack.JsLoaderRspackPlugin"
  }

  fn apply(&self, ctx: &mut ApplyContext) -> rspack_error::Result<()> {
    println!("apply start >>>>");

    ctx
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolve_loader::new(self));

    println!("apply end >>>>");
    Ok(())
  }
}

// #[plugin_hook(NormalModuleFactoryResolveLoader for JsLoaderRspackPlugin,stage = Compilation::OPTIMIZE_CHUNKS_STAGE_ADVANCED)]
#[plugin_hook(NormalModuleFactoryResolveLoader for JsLoaderRspackPlugin,stage = -1)]
pub(crate) async fn resolve_loader(
  &self,
  _context: &Context,
  _resolver: &Resolver,
  l: &ModuleRuleUseLoader,
) -> Result<Option<BoxLoader>> {
  // println!("----->");
  let loader_request = &l.loader;
  // println!("resolve_loader >>>> {}", loader_request);
  if loader_request.starts_with("builtin:test") {
    return Ok(get_builtin_test_loader(loader_request));
  }
  Ok(None)
}

pub fn get_builtin_test_loader(builtin: &str) -> Option<BoxLoader> {
  // if builtin.starts_with(rspack_loader_testing::SIMPLE_ASYNC_LOADER_IDENTIFIER) {
  //   return Some(Arc::new(rspack_loader_testing::SimpleAsyncLoader));
  // }
  // if builtin.starts_with(rspack_loader_testing::SIMPLE_LOADER_IDENTIFIER) {
  //   return Some(Arc::new(rspack_loader_testing::SimpleLoader));
  // }
  // if builtin.starts_with(rspack_loader_testing::PITCHING_LOADER_IDENTIFIER) {
  //   return Some(Arc::new(rspack_loader_testing::PitchingLoader));
  // }
  // if builtin.starts_with(rspack_loader_testing::PASS_THROUGH_LOADER_IDENTIFIER) {
  //   return Some(Arc::new(rspack_loader_testing::PassthroughLoader));
  // }
  // if builtin.starts_with(rspack_loader_testing::NO_PASS_THROUGH_LOADER_IDENTIFIER) {
  //   return Some(Arc::new(rspack_loader_testing::NoPassthroughLoader));
  // }
  // println!("get_builtin_test_loader>>>> {}", builtin);
  if builtin.starts_with(spack_loader_demo::SIMPLE_LOADER_IDENTIFIER) {
    return Some(Arc::new(spack_loader_demo::SimpleLoader));
  }
  None
}
