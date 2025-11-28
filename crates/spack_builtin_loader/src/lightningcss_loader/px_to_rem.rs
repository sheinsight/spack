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
  fn should_convert(&self) -> bool {
    if let Some(prop) = self.current_property.borrow().as_ref() {
      // 检查属性名是否在 prop_list 中
      // 支持通配符 '*' 表示所有属性
      self.options.prop_list.iter().any(|pattern| {
        if pattern == "*" {
          return true;
        }
        // 支持前缀匹配，如 "font" 匹配 "font-size", "font-weight" 等
        prop.starts_with(pattern.as_str()) || prop == pattern
      })
    } else {
      false
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
    for property in decls.iter_mut() {
      let property_name = match property {
        lightningcss::properties::Property::FontSize(_) => Some("font-size".to_string()),
        lightningcss::properties::Property::Font(_) => Some("font".to_string()),
        lightningcss::properties::Property::LineHeight(_) => Some("line-height".to_string()),
        lightningcss::properties::Property::LetterSpacing(_) => Some("letter-spacing".to_string()),
        lightningcss::properties::Property::WordSpacing(_) => Some("word-spacing".to_string()),
        _ => None,
      };

      *self.current_property.borrow_mut() = property_name;

      // 继续访问子节点（会调用 visit_length）
      property.visit_children(self)?;

      // 清除当前属性名
      *self.current_property.borrow_mut() = None;
    }
    Ok(())
  }

  fn visit_length(
    &mut self,
    length: &mut lightningcss::values::length::LengthValue,
  ) -> std::result::Result<(), Self::Error> {
    match length {
      lightningcss::values::length::LengthValue::Px(px) => {
        if !self.should_convert() {
          return Ok(());
        }
        if *px < self.options.min_pixel_value {
          return Ok(());
        }
        let rem_value = *px / self.options.root_value;
        let multiplier = 10_f32.powi(self.options.unit_precision);
        let rounded = (rem_value * multiplier).round() / multiplier;
        *length = lightningcss::values::length::LengthValue::Rem(rounded);
      }
      _ => {}
    }
    Ok(())
  }
}
