//! Quản lý Khung Nhóm Phân Vùng (Visual Group / Backdrop Box) kiểu ComfyUI.

use egui::{Color32, Pos2, Rect, Vec2};
use serde::{Deserialize, Serialize};

/// Khung phân vùng màu sắc bao quanh cụm Node trên Canvas (ComfyUI Style).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualGroup {
    pub id: usize,
    pub title: String,
    pub min: [f32; 2],
    pub max: [f32; 2],
    pub color_rgba: [u8; 4],
    #[serde(default)]
    pub is_locked: bool,
}

impl VisualGroup {
    /// Bảng màu mặc định xoay vòng dịu mắt theo chuẩn ComfyUI / Darkworks (alpha 35%).
    pub const PALETTE: &[[u8; 4]] = &[
        [138, 43, 226, 35],  // Purple
        [30, 144, 255, 35],  // Dodger Blue
        [0, 206, 209, 35],   // Dark Turquoise
        [46, 139, 87, 35],   // Sea Green
        [255, 140, 0, 35],   // Dark Orange
        [220, 20, 60, 35],   // Crimson
    ];

    /// Tên gọi trực quan của 6 màu trong bảng màu.
    pub const COLOR_NAMES: &[&str] = &[
        "Tím (Purple)",
        "Xanh dương (Blue)",
        "Xanh ngọc (Turquoise)",
        "Xanh lục (Sea Green)",
        "Cam (Orange)",
        "Đỏ thẫm (Crimson)",
    ];

    /// Khởi tạo một Visual Group mới với kích thước mặc định.
    #[must_use]
    pub fn new(id: usize, title: String, pos: Pos2, size: Vec2, color: Color32) -> Self {
        Self {
            id,
            title,
            min: [pos.x, pos.y],
            max: [pos.x + size.x, pos.y + size.y],
            color_rgba: [color.r(), color.g(), color.b(), color.a()],
            is_locked: false,
        }
    }

    /// Lấy vùng Rect của group trên canvas.
    #[must_use]
    pub fn rect(&self) -> Rect {
        Rect::from_min_max(
            Pos2::new(self.min[0], self.min[1]),
            Pos2::new(self.max[0], self.max[1]),
        )
    }

    /// Lấy vùng thanh tiêu đề (Header bar) của group (30px trên cùng).
    #[must_use]
    pub fn header_rect(&self) -> Rect {
        Rect::from_min_max(
            Pos2::new(self.min[0], self.min[1]),
            Pos2::new(self.max[0], self.min[1] + 30.0),
        )
    }

    /// Lấy vùng nút đóng/xóa nhanh "✖" ở góc trên bên phải thanh tiêu đề (24x24px).
    #[must_use]
    pub fn close_button_rect(&self) -> Rect {
        Rect::from_min_max(
            Pos2::new(self.max[0] - 28.0, self.min[1] + 3.0),
            Pos2::new(self.max[0] - 4.0, self.min[1] + 27.0),
        )
    }

    /// Lấy vùng kéo thanh tiêu đề (trừ đi phần nút đóng "✖" ở góc phải).
    #[must_use]
    pub fn header_drag_rect(&self) -> Rect {
        Rect::from_min_max(
            Pos2::new(self.min[0], self.min[1]),
            Pos2::new(self.max[0] - 30.0, self.min[1] + 30.0),
        )
    }

    /// Lấy vùng tay cầm co giãn (Resize Grip) góc dưới phải (32x32px, diện tích bắt chuột rộng rãi).
    #[must_use]
    pub fn resize_rect(&self) -> Rect {
        Rect::from_min_max(
            Pos2::new(self.max[0] - 32.0, self.max[1] - 32.0),
            Pos2::new(self.max[0], self.max[1]),
        )
    }

    /// Kiểm tra xem một group khác có nằm trọn 100% bên trong group này không (Strict Containment).
    #[must_use]
    pub fn contains_group(&self, other: &Self) -> bool {
        let r = self.rect();
        let other_r = other.rect();
        r.contains(other_r.min) && r.contains(other_r.max)
    }

    /// Kiểm tra một vị trí pos (ví dụ vị trí của một Node) có nằm bên trong group không.
    #[must_use]
    pub fn contains_node(&self, pos: Pos2) -> bool {
        self.rect().contains(pos)
    }

    /// Màu sắc mặc định xoay vòng dịu mắt theo chuẩn ComfyUI / Darkworks.
    #[must_use]
    pub fn default_color(index: usize) -> Color32 {
        let c = Self::PALETTE[index % Self::PALETTE.len()];
        Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
    }

    /// Thiết lập màu sắc trực tiếp cho group.
    pub fn set_color(&mut self, color: Color32) {
        self.color_rgba = [color.r(), color.g(), color.b(), color.a()];
    }

    /// Đổi màu sắc theo chỉ mục bảng màu xoay vòng.
    pub fn set_color_index(&mut self, index: usize) {
        let c = Self::PALETTE[index % Self::PALETTE.len()];
        self.color_rgba = c;
    }

    /// Bật/Tắt chế độ khóa vị trí (Lock Mode). Trả về trạng thái sau khi đổi.
    pub fn toggle_lock(&mut self) -> bool {
        self.is_locked = !self.is_locked;
        self.is_locked
    }

    /// Tự động căn chỉnh kích thước group để bao trọn các Node đang nằm trong group.
    /// Trả về `true` nếu có ít nhất 1 node và kích thước đã được cập nhật.
    pub fn auto_fit(&mut self, node_rects: impl IntoIterator<Item = Rect>) -> bool {
        let current_rect = self.rect();
        let mut bb = Rect::NOTHING;
        let mut count = 0;

        for r in node_rects {
            // Node được tính nếu nằm trong hoặc giao cắt với group
            if current_rect.intersects(r) || current_rect.contains(r.center()) {
                bb.extend_with(r.min);
                bb.extend_with(r.max);
                count += 1;
            }
        }

        if count > 0 && bb.is_finite() {
            // Mở rộng lề 35px thẩm mỹ và chừa 30px cho Header bar phía trên
            let min_x = bb.min.x - 35.0;
            let min_y = bb.min.y - 40.0;
            let max_x = (bb.max.x + 35.0).max(min_x + 220.0);
            let max_y = (bb.max.y + 35.0).max(min_y + 140.0);

            self.min = [min_x, min_y];
            self.max = [max_x, max_y];
            true
        } else {
            false
        }
    }

    /// Cập nhật tọa độ khi người dùng kéo di chuyển cả group.
    pub fn translate(&mut self, delta: Vec2) {
        if self.is_locked {
            return;
        }
        self.min[0] += delta.x;
        self.min[1] += delta.y;
        self.max[0] += delta.x;
        self.max[1] += delta.y;
    }

    /// Lấy màu sắc egui::Color32 của group.
    #[must_use]
    pub fn color(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(
            self.color_rgba[0],
            self.color_rgba[1],
            self.color_rgba[2],
            self.color_rgba[3],
        )
    }
}
