//! Kiểu dữ liệu Socket và bảng màu egui cho Node Graph.

use egui::Color32;

pub use darkworks_node_graph_core::NodeDataType;

/// Lấy màu sắc `egui::Color32` tương ứng với `NodeDataType`.
#[must_use]
pub const fn pin_color32(data_type: NodeDataType) -> Color32 {
    let rgb = data_type.color_rgb();
    Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}
