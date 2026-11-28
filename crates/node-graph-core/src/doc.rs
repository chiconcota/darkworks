//! Cấu trúc tài liệu đồ thị thuần dữ liệu (serializable trong `project.json` và `.preset.json`).

use serde::{Deserialize, Serialize};

use crate::node::DarkNode;
use crate::node_fx_types::TimeMode;

/// Gợi ý vị trí/thời lượng tự động khi tạo Overlay/FX Clip từ đồ thị Node Graph.
#[derive(Debug, Clone, PartialEq)]
pub struct OverlayPlacementHint {
    pub has_transition: bool,
    /// Thời lượng cố định (giây) theo `TimeDriver` (`Linear`/`SineLoop`/`PingPong`), `None` nếu là `Clip01` hoặc không có.
    pub driver_duration_sec: Option<f64>,
    pub has_clip01_driver: bool,
    /// Danh sách track nguồn (ví dụ `"V1"`, `"V2"`) nối vào hai cổng Input A/B của `Transition`.
    pub transition_tracks: Vec<String>,
}

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

    /// Lần ngược qua chuỗi `Reroute` để lấy node nguồn thực sự nối vào `(in_node, in_pin)`.
    #[must_use]
    pub fn resolve_source_node(&self, in_node: usize, in_pin: usize) -> Option<usize> {
        let (mut cur_id, _) = self.source_for_pin(in_node, in_pin)?;
        for _ in 0..32 {
            let entry = self.node(cur_id)?;
            if matches!(entry.node, DarkNode::Reroute { .. }) {
                cur_id = self.source_for_pin(cur_id, 0)?.0;
            } else {
                return Some(cur_id);
            }
        }
        None
    }

    fn upstream_video_track(&self, mut node_id: usize) -> Option<String> {
        for _ in 0..32 {
            let entry = self.node(node_id)?;
            match &entry.node {
                DarkNode::VideoInput { track_name } => return Some(track_name.clone()),
                DarkNode::Reroute { .. }
                | DarkNode::ColorGrade { .. }
                | DarkNode::BlurSharpen { .. }
                | DarkNode::DistortLens { .. }
                | DarkNode::Keyer { .. }
                | DarkNode::Transform2D { .. }
                | DarkNode::CropMask { .. }
                | DarkNode::CustomWgsl { .. } => {
                    node_id = self.source_for_pin(node_id, 0)?.0;
                }
                _ => return None,
            }
        }
        None
    }

    /// Phân tích đồ thị để gợi ý thời lượng & vùng overlap khi tạo FX Clip (`nodeGraph.createOverlayClip`).
    #[must_use]
    pub fn overlay_hint(&self) -> OverlayPlacementHint {
        let mut has_transition = false;
        let mut driver_duration_sec = None;
        let mut has_clip01_driver = false;
        let mut transition_tracks = Vec::new();

        let inspect_driver = |entry: &NodeEntry, dur_out: &mut Option<f64>, clip01_out: &mut bool| {
            if let DarkNode::TimeDriver { speed, mode, .. } = &entry.node {
                if *mode == TimeMode::Clip01 {
                    *clip01_out = true;
                } else if *speed > 1e-4 && dur_out.is_none() {
                    *dur_out = Some((1.0 / f64::from(*speed)).clamp(0.05, 600.0));
                }
            }
        };

        for entry in &self.nodes {
            if matches!(entry.node, DarkNode::Transition { .. }) {
                has_transition = true;
                for pin in [0, 1] {
                    if let Some((src_id, _)) = self.source_for_pin(entry.id, pin)
                        && let Some(track) = self.upstream_video_track(src_id)
                        && !transition_tracks.iter().any(|t: &String| t.eq_ignore_ascii_case(&track))
                    {
                        transition_tracks.push(track);
                    }
                }
                if let Some(driver_id) = self.resolve_source_node(entry.id, 2)
                    && let Some(driver_entry) = self.node(driver_id)
                {
                    inspect_driver(driver_entry, &mut driver_duration_sec, &mut has_clip01_driver);
                }
            }
        }

        if has_transition && driver_duration_sec.is_none() && !has_clip01_driver {
            for entry in &self.nodes {
                inspect_driver(entry, &mut driver_duration_sec, &mut has_clip01_driver);
            }
        }

        OverlayPlacementHint {
            has_transition,
            driver_duration_sec,
            has_clip01_driver,
            transition_tracks,
        }
    }
}
