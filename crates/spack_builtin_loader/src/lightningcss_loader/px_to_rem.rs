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
  // pub media_query: bool,
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
  fn should_convert(&self, property_name: Option<&str>) -> bool {
    if let Some(property_name) = property_name {
      self.options.prop_list.iter().any(|pattern| {
        if pattern == "*" {
          return true;
        }
        property_name.starts_with(pattern.as_str()) || property_name == pattern
      })
    } else {
      false
    }
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

  fn get_property_name(
    &self,
    property: &lightningcss::properties::Property,
  ) -> Option<&'static str> {
    match property {
      lightningcss::properties::Property::FontSize(_) => Some("font-size"),
      lightningcss::properties::Property::Font(_) => Some("font"),
      lightningcss::properties::Property::LineHeight(_) => Some("line-height"),
      lightningcss::properties::Property::LetterSpacing(_) => Some("letter-spacing"),
      lightningcss::properties::Property::WordSpacing(_) => Some("word-spacing"),
      _ => None,
    }
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
        let property_name = self.get_property_name(property).map(|s| s.to_string());

        *self.current_property.borrow_mut() = property_name;

        // 继续访问子节点（会调用 visit_length）
        property.visit_children(self)?;

        // 清除当前属性名
        *self.current_property.borrow_mut() = None;
      }
    } else {
      // 追加模式：先收集需要插入的声明，然后统一插入
      let mut properties_to_insert = Vec::new();

      for (index, property) in decls.declarations.iter().enumerate() {
        let property_name = self.get_property_name(property).map(|s| s.to_string());

        if self.should_convert(property_name.as_deref()) {
          let mut cloned = property.clone();

          // 设置当前属性名，转换克隆的 property 中的值
          *self.current_property.borrow_mut() = property_name;
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
          .map(|prop| self.should_convert(Some(prop.as_str())))
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
}
