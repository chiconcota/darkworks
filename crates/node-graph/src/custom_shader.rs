//! Xử lý Code Node: Custom WGSL JIT Shader Reflection & Syntax Validation.
//!
//! Cho phép người dùng viết code WGSL trực tiếp trong Node, tự động phân tích (reflect)
//! các biến `@group(..) @binding(..) var<uniform>` để sinh ra các cổng Socket điều khiển tương ứng.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::types::NodeDataType;

/// Lỗi phân tích cú pháp WGSL.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum WgslError {
    #[error("Lỗi cú pháp Shader WGSL: {0}")]
    SyntaxError(String),

    #[error("Thiếu hàm kết xuất chính @fragment fn fs_main()")]
    MissingFragmentEntry,
}

/// Thông tin về một biến Uniform được khai báo trong shader.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WgslUniform {
    /// Tên biến (ví dụ: `u_speed`, `u_color`, `u_intensity`).
    pub name: String,
    /// Chỉ số Group binding.
    pub group: u32,
    /// Chỉ số Binding index.
    pub binding: u32,
    /// Kiểu dữ liệu tương ứng trong hệ thống Socket.
    pub data_type: NodeDataType,
}

/// Bộ phân tích và kiểm định mã nguồn WGSL.
pub struct WgslParser;

impl WgslParser {
    /// Phân tích mã nguồn WGSL và trích xuất danh sách các biến Uniform.
    pub fn parse_uniforms(code: &str) -> Result<Vec<WgslUniform>, WgslError> {
        let mut uniforms = Vec::new();

        // 1. Kiểm tra sự tồn tại của Fragment Entry
        if !code.contains("fn fs_main") && !code.contains("@fragment") {
            return Err(WgslError::MissingFragmentEntry);
        }

        // 2. Tìm kiếm các khai báo uniform: `@group(X) @binding(Y) var<uniform> name: Type;`
        for line in code.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }

            if let Some(uniform) = Self::extract_uniform_from_line(trimmed) {
                uniforms.push(uniform);
            }
        }

        Ok(uniforms)
    }

    /// Trích xuất thông tin uniform từ 1 dòng code WGSL.
    fn extract_uniform_from_line(line: &str) -> Option<WgslUniform> {
        // Ví dụ: @group(0) @binding(1) var<uniform> u_intensity: f32;
        let group = Self::extract_attribute_val(line, "@group")?;
        let binding = Self::extract_attribute_val(line, "@binding")?;

        let var_part = line.split("var<uniform>").nth(1)?.trim();
        let name_type_split: Vec<&str> = var_part.split(':').collect();
        if name_type_split.len() < 2 {
            return Option::None;
        }

        let name = name_type_split[0].trim().to_string();
        let raw_type = name_type_split[1].trim().trim_end_matches(';').trim();

        let data_type = match raw_type {
            "f32" => NodeDataType::Scalar,
            "vec2<f32>" | "vec2f" => NodeDataType::Vec2,
            "vec4<f32>" | "vec4f" => NodeDataType::Color,
            _ => NodeDataType::Scalar,
        };

        Some(WgslUniform {
            name,
            group,
            binding,
            data_type,
        })
    }

    /// Trích xuất giá trị số nguyên bên trong thuộc tính @attribute(val).
    fn extract_attribute_val(line: &str, attr: &str) -> Option<u32> {
        let start_pos = line.find(attr)? + attr.len();
        let remainder = &line[start_pos..];
        let open_paren = remainder.find('(')?;
        let close_paren = remainder.find(')')?;

        let num_str = remainder[open_paren + 1..close_paren].trim();
        num_str.parse::<u32>().ok()
    }
}
