//! Định nghĩa kiểu dữ liệu socket và định tuyến tín hiệu trên Node Graph (thuần dữ liệu, không UI).

use serde::{Deserialize, Serialize};

/// Kiểu dữ liệu truyền qua các chân Socket trên Node Graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeDataType {
    /// Luồng Texture RGBA8/f32 GPU (Xanh lam Neon)
    Texture,
    /// Biên độ sóng âm Float f32 (Xanh lá)
    AudioFloat,
    /// Giá trị vô hướng Float f32 (Vàng)
    Scalar,
    /// Tọa độ 2 chiều Vec2 [f32; 2] (Cam)
    Vec2,
    /// Màu sắc Color [f32; 4] (Hồng cánh sen)
    Color,
    /// Tín hiệu kích hoạt lệnh tự động hóa ActionFlow (Tím)
    ActionFlow,
    /// Dữ liệu tổng quát bất kỳ (Trắng)
    Any,
}

impl NodeDataType {
    /// Màu RGB trực quan của chân cắm và dây nối.
    #[must_use]
    pub const fn color_rgb(&self) -> [u8; 3] {
        match self {
            Self::Texture => [0, 176, 255],    // Neon Blue
            Self::AudioFloat => [0, 230, 118], // Bright Green
            Self::Scalar => [255, 214, 0],     // Amber Yellow
            Self::Vec2 => [255, 145, 0],       // Deep Orange
            Self::Color => [255, 64, 129],     // Pink Rose
            Self::ActionFlow => [170, 0, 255], // Electric Purple
            Self::Any => [220, 220, 220],      // Light Gray
        }
    }

    /// Tên hiển thị ngắn gọn của kiểu dữ liệu.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Texture => "Texture",
            Self::AudioFloat => "Audio",
            Self::Scalar => "Value",
            Self::Vec2 => "Vec2",
            Self::Color => "Color",
            Self::ActionFlow => "Action",
            Self::Any => "Any",
        }
    }

    /// Kiểm tra 2 kiểu dữ liệu có thể kết nối với nhau không.
    #[must_use]
    pub const fn can_connect_to(&self, target: Self) -> bool {
        match (*self, target) {
            (Self::Any, _) | (_, Self::Any) => true,
            (Self::AudioFloat, Self::Scalar) | (Self::Scalar, Self::AudioFloat) => true,
            (a, b) => (a as usize) == (b as usize),
        }
    }
}
