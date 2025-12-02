use std::cell::RefCell;

use lightningcss::visitor::{Visit, VisitTypes, Visitor};
use rspack_cacheable::cacheable;
use serde::{Deserialize, Serialize};

#[cacheable]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PxToRemOpts {
  pub root_value: f32,
  pub unit_precision: i32,
  pub prop_list: Vec<String>,
  pub replace: bool,
  pub media_query: bool,
  pub min_pixel_value: f32,
  // pub exclude: Vec<String>,
  // pub unit: String,
}

pub struct PxToRemVisitor {
  pub options: PxToRemOpts,
  current_property: RefCell<Option<String>>,
}

impl PxToRemVisitor {
  pub fn new(options: PxToRemOpts) -> Self {
    Self {
      options,
      current_property: RefCell::new(None),
    }
  }

  // 辅助方法：检查当前属性是否应该转换
  fn should_convert(&self, property_name: &str) -> bool {
    self.options.prop_list.iter().any(|pattern| {
      if pattern == "*" {
        return true;
      }
      property_name.starts_with(pattern.as_str()) || property_name == pattern
    })
  }

  // 转换 length 值
  fn convert_px_to_rem(&self, px: f32) -> Option<f32> {
    if px < self.options.min_pixel_value {
      return None;
    }
    let rem_value = px / self.options.root_value;
    let multiplier = 10_f32.powi(self.options.unit_precision);
    let rounded = (rem_value * multiplier).round() / multiplier;
    Some(rounded)
  }

  // 处理 media condition（媒体查询条件）
  // 这是递归处理的核心，因为条件可以嵌套（and、or、not 等逻辑运算符）
  fn process_media_condition(
    &mut self,
    condition: &mut lightningcss::media_query::MediaCondition,
  ) -> std::result::Result<(), ()> {
    use lightningcss::media_query::MediaCondition;

    match condition {
      // Feature 类型：单个特性，如 (min-width: 768px)
      MediaCondition::Feature(feature) => {
        self.process_media_feature(feature)?;
      }
      // Not 类型：否定条件，如 not (min-width: 768px)
      MediaCondition::Not(inner_condition) => {
        self.process_media_condition(inner_condition)?;
      }
      // Operation 类型：多个条件用 and/or 连接
      // 例如：(min-width: 768px) and (max-width: 1024px)
      MediaCondition::Operation { conditions, .. } => {
        for cond in conditions.iter_mut() {
          self.process_media_condition(cond)?;
        }
      }
    }

    Ok(())
  }

  // 处理 media feature（媒体特性）
  // 这是实际包含尺寸值的地方，如 min-width: 768px
  fn process_media_feature(
    &mut self,
    feature: &mut lightningcss::media_query::MediaFeature,
  ) -> std::result::Result<(), ()> {
    use lightningcss::media_query::QueryFeature;

    match feature {
      // Plain 类型：标准特性，如 (width: 768px) 或 (min-width: 768px)
      QueryFeature::Plain { value, .. } => {
        self.process_media_feature_value(value)?;
      }
      // Range 类型：范围特性，如 (width > 768px)
      QueryFeature::Range { value, .. } => {
        self.process_media_feature_value(value)?;
      }
      // Interval 类型：区间特性，如 (400px < width < 1000px)
      QueryFeature::Interval { start, end, .. } => {
        // 处理起始值和结束值
        self.process_media_feature_value(start)?;
        self.process_media_feature_value(end)?;
      }
      // Boolean 类型：布尔特性，如 (hover)，不包含值，跳过
      QueryFeature::Boolean { .. } => {}
    }

    Ok(())
  }

  // 处理 media feature value（媒体特性的值）
  // 这里处理具体的长度值，将 px 转换为 rem
  fn process_media_feature_value(
    &mut self,
    value: &mut lightningcss::media_query::MediaFeatureValue,
  ) -> std::result::Result<(), ()> {
    use lightningcss::media_query::MediaFeatureValue;
    use lightningcss::values::length::Length;

    match value {
      // Length 类型：长度值，如 768px
      // MediaFeatureValue::Length 包含的是 Length enum
      MediaFeatureValue::Length(media_length) => {
        // Length 有两个变体：Value(LengthValue) 和 Calc(...)
        // 我们只处理 Value 的情况
        match media_length {
          Length::Value(length) => {
            // 在 media query 中，我们总是转换，不检查 prop_list
            // 因为 media query 不是 CSS 属性，而是查询条件
            // 所以我们临时设置一个标记，让 visit_length 知道这是 media query 上下文
            let old_property = self.current_property.replace(Some("*".to_string()));
            self.visit_length(length)?;
            *self.current_property.borrow_mut() = old_property;
          }
          // Calc 类型暂时不处理，因为计算表达式比较复杂
          Length::Calc(_) => {}
        }
      }
      // Ratio 类型：比例值，如 16/9，不需要转换
      // Number 类型：数字值，不需要转换
      // Integer 类型：整数值，不需要转换
      // Boolean 类型：布尔值，不需要转换
      // Resolution 类型：分辨率值，如 300dpi，不需要转换
      // Ident 类型：标识符，不需要转换
      // Env 类型：环境变量，不需要转换
      // 其他类型都不需要处理
      _ => {}
    }

    Ok(())
  }
}

impl<'i> Visitor<'i> for PxToRemVisitor {
  type Error = ();

  fn visit_types(&self) -> VisitTypes {
    VisitTypes::all()
  }

  fn visit_declaration_block(
    &mut self,
    decls: &mut lightningcss::declaration::DeclarationBlock<'i>,
  ) -> std::result::Result<(), Self::Error> {
    if self.options.replace {
      for property in decls.iter_mut() {
        let property_name = property.property_id().name().to_string();

        *self.current_property.borrow_mut() = Some(property_name);

        // 继续访问子节点（会调用 visit_length）
        property.visit_children(self)?;

        // 清除当前属性名
        *self.current_property.borrow_mut() = None;
      }
    } else {
      // 追加模式：先收集需要插入的声明，然后统一插入
      let mut properties_to_insert = Vec::new();

      for (index, property) in decls.declarations.iter().enumerate() {
        let property_name = property.property_id().name().to_string();

        if self.should_convert(&property_name) {
          let mut cloned = property.clone();

          // 设置当前属性名，转换克隆的 property 中的值
          *self.current_property.borrow_mut() = Some(property_name);
          cloned.visit_children(self)?; // 这里会把克隆体的 px 转成 rem
          *self.current_property.borrow_mut() = None;

          // 记录需要插入的位置和声明
          properties_to_insert.push((index + 1, cloned));
        }
      }

      // 从后往前插入，避免索引偏移问题
      for (insert_index, property) in properties_to_insert.into_iter().rev() {
        decls.declarations.insert(insert_index, property);
      }
    }

    Ok(())
  }

  fn visit_length(
    &mut self,
    length: &mut lightningcss::values::length::LengthValue,
  ) -> std::result::Result<(), Self::Error> {
    match length {
      lightningcss::values::length::LengthValue::Px(px) => {
        // 检查当前属性是否需要转换
        let should_convert = self
          .current_property
          .borrow()
          .as_ref()
          .map(|prop| self.should_convert(prop.as_str()))
          .unwrap_or(false);

        if !should_convert {
          return Ok(());
        }

        if let Some(rem_value) = self.convert_px_to_rem(*px) {
          *length = lightningcss::values::length::LengthValue::Rem(rem_value);
        }
      }
      _ => {}
    }
    Ok(())
  }

  // 访问 media query 列表
  // 这是 CSS @media 规则的入口点，例如：@media (min-width: 768px) { ... }
  fn visit_media_list(
    &mut self,
    media: &mut lightningcss::media_query::MediaList<'i>,
  ) -> std::result::Result<(), Self::Error> {
    // 如果配置中没有启用 media_query 转换，直接跳过
    if !self.options.media_query {
      return Ok(());
    }

    // 遍历所有的 media query（可能有多个，用逗号分隔）
    // 例如：@media (min-width: 768px), (max-width: 1024px)
    for query in media.media_queries.iter_mut() {
      // media query 包含 condition（条件表达式）
      // 例如：(min-width: 768px) 就是一个 condition
      if let Some(condition) = &mut query.condition {
        self.process_media_condition(condition)?;
      }
    }

    Ok(())
  }
}
