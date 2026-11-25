//! Cấu trúc tài liệu đồ thị thuần dữ liệu (serializable trong `project.json` và `.preset.json`).

use serde::{Deserialize, Serialize};

use crate::node::DarkNode;

/// Một Node đã được đặt lên đồ thị cùng tọa độ và tên tùy chỉnh.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeEntry {
    pub id: usize,
    pub pos: [f32; 2],
    pub node: DarkNode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_title: Option<String>,
}

/// Một dây nối có hướng giữa 2 cổng Socket trên đồ thị.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeWire {
    pub out_node: usize,
    pub out_pin: usize,
    pub in_node: usize,
    pub in_pin: usize,
}

/// Dữ liệu thuần của khung phân vùng màu (Visual Group) không phụ thuộc `egui`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualGroupDoc {
    pub id: usize,
    pub title: String,
    pub min: [f32; 2],
    pub max: [f32; 2],
    pub color_rgba: [u8; 4],
    #[serde(default)]
    pub is_locked: bool,
}

/// Biểu diễn đầy đủ một đồ thị DAG để lưu trong `TrackItem` (`project.json`) và thực thi bởi `crates/render`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NodeGraphDoc {
    pub nodes: Vec<NodeEntry>,
    pub wires: Vec<NodeWire>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<VisualGroupDoc>,
}

impl NodeGraphDoc {
    /// Tạo tài liệu đồ thị rỗng.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Kiểm tra đồ thị có chứa ít nhất một Node không.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Tìm node theo `id`.
    #[must_use]
    pub fn node(&self, id: usize) -> Option<&NodeEntry> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Tìm tất cả các dây nối đi vào một `in_node`, sắp xếp theo `in_pin`.
    #[must_use]
    pub fn incoming_wires(&self, in_node: usize) -> Vec<NodeWire> {
        let mut list: Vec<NodeWire> = self.wires.iter().copied().filter(|w| w.in_node == in_node).collect();
        list.sort_by_key(|w| w.in_pin);
        list
    }

    /// Tìm nguồn nối vào đúng `(in_node, in_pin)`.
    #[must_use]
    pub fn source_for_pin(&self, in_node: usize, in_pin: usize) -> Option<(usize, usize)> {
        self.wires
            .iter()
            .find(|w| w.in_node == in_node && w.in_pin == in_pin)
            .map(|w| (w.out_node, w.out_pin))
    }
}
