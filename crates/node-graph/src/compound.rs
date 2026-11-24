//! Module quản lý Black Node (Compound Node / Subgraph).
//!
//! Cho phép đóng gói một cụm nhiều Atomic Nodes thành 1 Node hộp đen duy nhất,
//! mở rộng đồ thị con (drill-down), bộc lộ các cổng Socket cần thiết ra ngoài (Exposed Pins),
//! và xuất/nạp Preset `.preset.json`.

use egui_snarl::Snarl;
use serde::{Deserialize, Serialize};

use crate::node::DarkNode;
use crate::preset::NodeGraphPreset;
use crate::types::NodeDataType;

/// Định nghĩa một chân cắm được bộc lộ ra ngoài của Black Node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExposedPin {
    /// Tên hiển thị của pin trên vỏ Black Node bên ngoài.
    pub label: String,
    /// Kiểu dữ liệu của socket.
    pub data_type: NodeDataType,
    /// ID của node nội bộ sở hữu socket này.
    pub internal_node_idx: usize,
    /// Chỉ số pin nội bộ.
    pub internal_pin_idx: usize,
    /// Là cổng Input (true) hay Output (false).
    pub is_input: bool,
}

/// Dữ liệu hoàn chỉnh của một Black Node (Compound Subgraph).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackNodeSubgraph {
    /// Tên định danh của Black Node (ví dụ: "News Ticker", "Neon Glow FX").
    pub name: String,
    /// Đồ thị con bên trong chứa các Atomic Nodes.
    pub inner_snarl: Snarl<DarkNode>,
    /// Danh sách các cổng Input bộc lộ ra ngoài vỏ.
    pub exposed_inputs: Vec<ExposedPin>,
    /// Danh sách các cổng Output bộc lộ ra ngoài vỏ.
    pub exposed_outputs: Vec<ExposedPin>,
}

impl BlackNodeSubgraph {
    /// Tạo một Black Node rỗng mới.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            inner_snarl: Snarl::new(),
            exposed_inputs: Vec::new(),
            exposed_outputs: Vec::new(),
        }
    }

    /// Đóng gói Black Node thành Preset JSON để lưu vào ổ đĩa.
    pub fn to_preset_json(&self) -> Result<String, serde_json::Error> {
        let preset = NodeGraphPreset::from_snarl(format!("BlackNode: {}", self.name), &self.inner_snarl);
        serde_json::to_string_pretty(&preset)
    }

    /// Nạp Black Node từ Preset JSON.
    pub fn from_preset_json(name: impl Into<String>, json_str: &str) -> Result<Self, serde_json::Error> {
        let preset: NodeGraphPreset = serde_json::from_str(json_str)?;
        let mut inner_snarl = Snarl::new();
        preset.apply_to_snarl(&mut inner_snarl);

        let mut subgraph = Self::new(name);
        subgraph.inner_snarl = inner_snarl;
        subgraph.auto_detect_exposed_pins();
        Ok(subgraph)
    }

    /// Tự động dò tìm các chân cắm chưa kết nối hoặc các cổng Input/Output đầu mối để bộc lộ ra ngoài vỏ.
    pub fn auto_detect_exposed_pins(&mut self) {
        self.exposed_inputs.clear();
        self.exposed_outputs.clear();

        for (node_id, node) in self.inner_snarl.node_ids() {
            match node {
                DarkNode::VideoInput { track_name } => {
                    self.exposed_outputs.push(ExposedPin {
                        label: format!("{track_name} Out"),
                        data_type: NodeDataType::Texture,
                        internal_node_idx: node_id.0,
                        internal_pin_idx: 0,
                        is_input: false,
                    });
                }
                DarkNode::Output => {
                    self.exposed_inputs.push(ExposedPin {
                        label: "Main Texture In".to_string(),
                        data_type: NodeDataType::Texture,
                        internal_node_idx: node_id.0,
                        internal_pin_idx: 0,
                        is_input: true,
                    });
                }
                DarkNode::Transform2D { .. } => {
                    self.exposed_inputs.push(ExposedPin {
                        label: "Transform In".to_string(),
                        data_type: NodeDataType::Texture,
                        internal_node_idx: node_id.0,
                        internal_pin_idx: 0,
                        is_input: true,
                    });
                }
                _ => {}
            }
        }
    }

    /// Bộc lộ thủ công một Pin ra ngoài vỏ Black Node.
    pub fn expose_pin(
        &mut self,
        label: impl Into<String>,
        data_type: NodeDataType,
        internal_node_idx: usize,
        internal_pin_idx: usize,
        is_input: bool,
    ) {
        let pin = ExposedPin {
            label: label.into(),
            data_type,
            internal_node_idx,
            internal_pin_idx,
            is_input,
        };

        if is_input {
            self.exposed_inputs.push(pin);
        } else {
            self.exposed_outputs.push(pin);
        }
    }
}
