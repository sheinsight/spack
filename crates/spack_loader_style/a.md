⏺ 在 rspack 的 Rust loader 中，获取类似 "css-loader!./src/style.css" 这样的 loader chain 数据有以下几种方式：

1. 主要获取方法

async fn run(&self, loader_context: &mut LoaderContext<Context>) -> Result<()> {
// 获取剩余请求 (当前 loader 之后的 loader chain + 资源)
let remaining_request = loader_context.remaining_request()
.display_with_suffix(loader_context.resource());

      // 获取之前请求 (当前 loader 之前的 loader chain)
      let previous_request = loader_context.previous_request().to_string();

      // 获取当前请求 (从当前 loader 开始的 chain)
      let current_request = loader_context.current_request()
          .display_with_suffix(loader_context.resource());

      // 获取完整请求 (整个 loader chain)
      let full_request = loader_context.request()
          .display_with_suffix(loader_context.resource());

      println!("remaining: {}", remaining_request);  // css-loader!./src/style.css
      println!("previous: {}", previous_request);    // style-loader
      println!("current: {}", current_request);      // css-loader!./src/style.css
      println!("full: {}", full_request);           // style-loader!css-loader!./src/style.css

}

2. 完整示例

use rspack_loader_runner::{Loader, LoaderContext, Identifier, Result};
use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};

#[cacheable]
pub struct RequestAwareLoader;

#[cacheable_dyn] #[async_trait]
impl Loader<RunnerContext> for RequestAwareLoader {
fn identifier(&self) -> Identifier {
"builtin:request-aware-loader".into()
}

      async fn pitch(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
          // 在 pitch 阶段获取请求信息
          let remaining_request = loader_context.remaining_request()
              .display_with_suffix(loader_context.resource());
          let previous_request = loader_context.previous_request().to_string();

          println!("Pitch - remaining: {}", remaining_request);
          println!("Pitch - previous: {}", previous_request);

          // 如果需要，可以在 pitch 阶段直接返回处理结果
          if remaining_request.contains("css-loader") {
              loader_context.finish_with(format!(
                  "module.exports = require({});",
                  serde_json::to_string(&format!("!!{}", remaining_request))?
              ));
          }

          Ok(())
      }

      async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
          // 获取各种请求信息
          let remaining_request = loader_context.remaining_request()
              .display_with_suffix(loader_context.resource());
          let previous_request = loader_context.previous_request().to_string();
          let current_request = loader_context.current_request()
              .display_with_suffix(loader_context.resource());

          // 处理内容
          let Some(content) = loader_context.take_content() else {
              return Ok(());
          };

          let processed = format!(
              r#"

// Loader chain information:
// Previous: {}
// Current: {}
// Remaining: {}

{}
"#,
previous_request,
current_request,
remaining_request,
content.try_into_string()?
);

          loader_context.finish_with(processed);
          Ok(())
      }

}

3. LoaderItemList 的核心方法

impl<Context: Send> LoaderContext<Context> {
// 获取剩余的 loader (当前之后)
pub fn remaining*request(&self) -> LoaderItemList<'*, Context>

      // 获取之前的 loader (当前之前)
      pub fn previous_request(&self) -> LoaderItemList<'_, Context>

      // 获取当前请求 (从当前开始)
      pub fn current_request(&self) -> LoaderItemList<'_, Context>

      // 获取完整请求 (所有 loader)
      pub fn request(&self) -> LoaderItemList<'_, Context>

}

// LoaderItemList 提供的方法
impl LoaderItemList {
// 转为字符串 (用 "!" 连接)
fn to_string() -> String

      // 添加后缀 (通常是资源路径)
      fn display_with_suffix(&self, suffix: &str) -> String

}

4. 实际使用场景

对于 loader chain: style-loader!css-loader!sass-loader!./src/style.scss

当执行到 css-loader 时：

async fn run(&self, loader_context: &mut LoaderContext<Context>) -> Result<()> {
let remaining = loader_context.remaining_request()
.display_with_suffix(loader_context.resource());
// remaining = "sass-loader!./src/style.scss"

      let previous = loader_context.previous_request().to_string();
      // previous = "style-loader"

      let current = loader_context.current_request()
          .display_with_suffix(loader_context.resource());
      // current = "css-loader!sass-loader!./src/style.scss"

      // 可以基于这些信息做条件处理
      if remaining.contains("sass-loader") {
          // 处理 SASS 相关逻辑
      }

}

5. 核心要点

- 字符串格式: loader 之间用 ! 分隔，如 "css-loader!./src/style.css"
- 动态获取: 通过 LoaderContext 的方法动态获取不同范围的 loader chain
- 带资源路径: 使用 display_with_suffix() 方法添加资源路径
- 实时更新: 在不同 loader 执行时，这些值会自动更新反映当前状态

这样就可以在 Rust loader 中获取和处理完整的 loader chain 信息了。
