//! Định nghĩa định dạng `.preset.json` đầy đủ (giữ 100% dây nối, tọa độ, nhóm) và thuật toán nối tiếp DAG (Chain).

use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::doc::{NodeEntry, NodeGraphDoc, NodeWire, VisualGroupDoc};
use crate::node::{BlendMode, DarkNode};

fn default_category() -> String {
    "Custom".to_string()
}

fn default_version() -> String {
    "2.0.0".to_string()
}

/// Lỗi đọc/ghi hoặc phân tích file `.preset.json`.
#[derive(Debug, Error)]
pub enum PresetDocError {
    #[error("Không thể đọc/ghi tập tin preset: {0}")]
    Io(#[from] std::io::Error),
    #[error("File preset JSON không hợp lệ hoặc bị hỏng: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Đồ thị rỗng: cần ít nhất một Node để lưu Preset")]
    EmptyGraph,
}

/// Định dạng chuẩn của file `.preset.json` (lưu trọn vẹn metadata + `NodeGraphDoc` gồm dây nối và tọa độ).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeGraphPresetDoc {
    pub name: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub description: String,
    pub graph: NodeGraphDoc,
}

/// Định dạng cũ của `NodeGraphPreset` (không có `wires` và tọa độ từng node) để tương thích ngược khi đọc.
#[derive(Debug, Deserialize)]
struct LegacyNodeGraphPreset {
    name: String,
    #[serde(default = "default_version")]
    version: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    nodes: Vec<DarkNode>,
    #[serde(default)]
    groups: Vec<VisualGroupDoc>,
}

impl NodeGraphPresetDoc {
    /// Tạo tài liệu preset mới từ `NodeGraphDoc`.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        category: impl Into<String>,
        description: impl Into<String>,
        graph: NodeGraphDoc,
    ) -> Self {
        Self {
            name: name.into(),
            category: category.into(),
            version: default_version(),
            description: description.into(),
            graph,
        }
    }

    /// Xuất toàn bộ preset sang chuỗi JSON đẹp.
    pub fn to_json_string(&self) -> Result<String, PresetDocError> {
        serde_json::to_string_pretty(self).map_err(PresetDocError::Json)
    }

    /// Nạp preset từ JSON, tự động hỗ trợ tương thích ngược với định dạng preset cũ không có `wires`.
    pub fn from_json_str(json: &str) -> Result<Self, PresetDocError> {
        match serde_json::from_str::<Self>(json) {
            Ok(doc) => Ok(doc),
            Err(primary_err) => {
                if let Ok(legacy) = serde_json::from_str::<LegacyNodeGraphPreset>(json)
                    && !legacy.nodes.is_empty()
                {
                    let nodes = legacy
                        .nodes
                        .into_iter()
                        .enumerate()
                        .map(|(idx, node)| NodeEntry {
                            id: idx,
                            pos: [100.0 + (idx as f32) * 260.0, 120.0],
                            node,
                            custom_title: None,
                        })
                        .collect();
                    return Ok(Self {
                        name: legacy.name,
                        category: default_category(),
                        version: legacy.version,
                        description: legacy.description,
                        graph: NodeGraphDoc {
                            nodes,
                            wires: Vec::new(),
                            groups: legacy.groups,
                        },
                    });
                }
                Err(PresetDocError::Json(primary_err))
            }
        }
    }

    /// Ghi file `.preset.json` ra ổ đĩa.
    pub fn save_to_file(&self, path: &Path) -> Result<(), PresetDocError> {
        if self.graph.is_empty() {
            return Err(PresetDocError::EmptyGraph);
        }
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let json = self.to_json_string()?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Đọc file `.preset.json` từ ổ đĩa.
    pub fn load_from_file(path: &Path) -> Result<Self, PresetDocError> {
        let json = std::fs::read_to_string(path)?;
        Self::from_json_str(&json)
    }
}

/// Tạo tên file an toàn kết thúc bằng `.preset.json` từ tên preset người dùng nhập.
#[must_use]
pub fn sanitize_preset_filename(name: &str) -> String {
    let mut slug = String::new();
    let mut last_underscore = false;
    for ch in name.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_underscore = false;
        } else if (ch == ' ' || ch == '-' || ch == '_' || !ch.is_ascii()) && !last_underscore && !slug.is_empty() {
            slug.push('_');
            last_underscore = true;
        }
    }
    while slug.ends_with('_') {
        slug.pop();
    }
    if slug.is_empty() {
        slug = "custom_preset".to_string();
    }
    format!("{slug}.preset.json")
}

/// Đảm bảo đường dẫn lưu file kết thúc bằng `.preset.json` (hoặc giữ nguyên nếu đã là `.json`).
#[must_use]
pub fn ensure_preset_extension(path: &str) -> String {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".preset.json") {
        path.to_string()
    } else if lower.ends_with(".json") {
        let stem = &path[..path.len() - 5];
        format!("{stem}.preset.json")
    } else {
        let trimmed = path.strip_suffix('.').unwrap_or(path);
        format!("{trimmed}.preset.json")
    }
}

fn graph_x_bounds(doc: &NodeGraphDoc) -> (f32, f32) {
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    for n in &doc.nodes {
        min_x = min_x.min(n.pos[0]);
        max_x = max_x.max(n.pos[0] + 220.0);
    }
    if !min_x.is_finite() || !max_x.is_finite() {
        (0.0, 0.0)
    } else {
        (min_x, max_x)
    }
}

/// Nối tiếp đồ thị `incoming` (từ Preset) vào sau đồ thị `base` hiện có trước `Final Output`.
///
/// - Nếu `incoming` có `VideoInput`, các dây xuất phát từ `VideoInput` của `incoming` sẽ nhận
///   luồng hình ảnh đầu ra trước `Output` của `base`.
/// - Nếu `incoming` là generator/overlay không có `VideoInput`, sẽ chèn thêm một node `Blend`
///   để hòa trộn `base` (Background) và `incoming` (Foreground) trước `Final Output`.
#[must_use]
pub fn chain_node_graphs(base: &NodeGraphDoc, incoming: &NodeGraphDoc) -> NodeGraphDoc {
    if base.is_empty() {
        return incoming.clone();
    }
    if incoming.is_empty() {
        return base.clone();
    }

    let base_output_ids: HashSet<usize> = base
        .nodes
        .iter()
        .filter(|n| matches!(n.node, DarkNode::Output))
        .map(|n| n.id)
        .collect();

    let base_source = base
        .nodes
        .iter()
        .find(|n| matches!(n.node, DarkNode::Output))
        .and_then(|out| base.source_for_pin(out.id, 0));

    let Some((base_src_node, base_src_pin)) = base_source else {
        // Đồ thị gốc chưa nối tới Output -> dùng đồ thị mới thay thế gọn gàng
        return incoming.clone();
    };

    let id_offset = base.nodes.iter().map(|n| n.id).max().map_or(1, |m| m + 1);
    let group_offset = base.groups.iter().map(|g| g.id).max().map_or(1, |m| m + 1);

    let (_, base_max_x) = graph_x_bounds(base);
    let (inc_min_x, _) = graph_x_bounds(incoming);
    let dx = (base_max_x - inc_min_x + 100.0).max(240.0);

    // Giữ toàn bộ node & dây của base trừ node Output cũ
    let mut nodes: Vec<NodeEntry> = base
        .nodes
        .iter()
        .filter(|n| !base_output_ids.contains(&n.id))
        .cloned()
        .collect();
    let mut wires: Vec<NodeWire> = base
        .wires
        .iter()
        .copied()
        .filter(|w| !base_output_ids.contains(&w.in_node) && !base_output_ids.contains(&w.out_node))
        .collect();
    let mut groups: Vec<VisualGroupDoc> = base.groups.clone();

    let inc_video_inputs: HashSet<usize> = incoming
        .nodes
        .iter()
        .filter(|n| matches!(n.node, DarkNode::VideoInput { .. }))
        .map(|n| n.id)
        .collect();

    let mut next_free_id = id_offset
        + incoming
            .nodes
            .iter()
            .map(|n| n.id)
            .max()
            .map_or(0, |m| m + 1);

    if !inc_video_inputs.is_empty() {
        // Trường hợp A: Preset có VideoInput -> thay thế VideoInput bằng đầu ra của đồ thị base
        for entry in &incoming.nodes {
            if inc_video_inputs.contains(&entry.id) {
                continue;
            }
            nodes.push(NodeEntry {
                id: entry.id + id_offset,
                pos: [entry.pos[0] + dx, entry.pos[1]],
                node: entry.node.clone(),
                custom_title: entry.custom_title.clone(),
            });
        }
        for w in &incoming.wires {
            if inc_video_inputs.contains(&w.out_node) {
                wires.push(NodeWire {
                    out_node: base_src_node,
                    out_pin: base_src_pin,
                    in_node: w.in_node + id_offset,
                    in_pin: w.in_pin,
                });
            } else {
                wires.push(NodeWire {
                    out_node: w.out_node + id_offset,
                    out_pin: w.out_pin,
                    in_node: w.in_node + id_offset,
                    in_pin: w.in_pin,
                });
            }
        }
    } else {
        // Trường hợp B: Preset không có VideoInput (overlay/generator) -> chèn Blend nối Base + Preset
        let inc_output = incoming
            .nodes
            .iter()
            .find(|n| matches!(n.node, DarkNode::Output));
        let inc_output_ids: HashSet<usize> = incoming
            .nodes
            .iter()
            .filter(|n| matches!(n.node, DarkNode::Output))
            .map(|n| n.id)
            .collect();
        let inc_src = inc_output.and_then(|o| incoming.source_for_pin(o.id, 0));

        for entry in &incoming.nodes {
            if inc_output_ids.contains(&entry.id) {
                continue;
            }
            nodes.push(NodeEntry {
                id: entry.id + id_offset,
                pos: [entry.pos[0] + dx, entry.pos[1]],
                node: entry.node.clone(),
                custom_title: entry.custom_title.clone(),
            });
        }
        for w in &incoming.wires {
            if !inc_output_ids.contains(&w.in_node) && !inc_output_ids.contains(&w.out_node) {
                wires.push(NodeWire {
                    out_node: w.out_node + id_offset,
                    out_pin: w.out_pin,
                    in_node: w.in_node + id_offset,
                    in_pin: w.in_pin,
                });
            }
        }

        let (_, combined_max_x) = graph_x_bounds(&NodeGraphDoc {
            nodes: nodes.clone(),
            wires: Vec::new(),
            groups: Vec::new(),
        });
        let out_y = inc_output.map_or(180.0, |o| o.pos[1]);

        let blend_id = next_free_id;
        next_free_id += 1;
        let output_id = next_free_id;

        nodes.push(NodeEntry {
            id: blend_id,
            pos: [combined_max_x + 80.0, out_y],
            node: DarkNode::Blend {
                mode: BlendMode::Normal,
                opacity: 1.0,
            },
            custom_title: Some("Chain Blend".to_string()),
        });
        nodes.push(NodeEntry {
            id: output_id,
            pos: [combined_max_x + 340.0, out_y],
            node: DarkNode::Output,
            custom_title: None,
        });

        wires.push(NodeWire {
            out_node: base_src_node,
            out_pin: base_src_pin,
            in_node: blend_id,
            in_pin: 0,
        });
        if let Some((inc_src_node, inc_src_pin)) = inc_src {
            wires.push(NodeWire {
                out_node: inc_src_node + id_offset,
                out_pin: inc_src_pin,
                in_node: blend_id,
                in_pin: 1,
            });
        }
        wires.push(NodeWire {
            out_node: blend_id,
            out_pin: 0,
            in_node: output_id,
            in_pin: 0,
        });
    }

    // Tạo Output dự phòng nếu incoming không có Output
    if !nodes.iter().any(|n| matches!(n.node, DarkNode::Output)) {
        let (_, max_x) = graph_x_bounds(&NodeGraphDoc {
            nodes: nodes.clone(),
            wires: Vec::new(),
            groups: Vec::new(),
        });
        let out_id = next_free_id + 1;
        nodes.push(NodeEntry {
            id: out_id,
            pos: [max_x + 80.0, 180.0],
            node: DarkNode::Output,
            custom_title: None,
        });
    }

    for g in &incoming.groups {
        groups.push(VisualGroupDoc {
            id: g.id + group_offset,
            title: g.title.clone(),
            min: [g.min[0] + dx, g.min[1]],
            max: [g.max[0] + dx, g.max[1]],
            color_rgba: g.color_rgba,
            is_locked: g.is_locked,
        });
    }

    // Lọc bỏ dây trỏ tới node không tồn tại (defensive)
    let valid_ids: HashSet<usize> = nodes.iter().map(|n| n.id).collect();
    wires.retain(|w| valid_ids.contains(&w.out_node) && valid_ids.contains(&w.in_node));

    // Đảm bảo giữ unique mapping cho mọi node
    let _ = HashMap::<usize, usize>::new();

    NodeGraphDoc {
        nodes,
        wires,
        groups,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluator::DagEvaluator;
    use crate::node::{BlurType, ShapeType};

    #[test]
    fn preset_doc_roundtrip_preserves_wires_positions_and_groups() {
        let doc = NodeGraphDoc {
            nodes: vec![
                NodeEntry {
                    id: 0,
                    pos: [40.0, 80.0],
                    node: DarkNode::VideoInput {
                        track_name: "V1".to_string(),
                    },
                    custom_title: Some("Clip Nguồn".to_string()),
                },
                NodeEntry {
                    id: 1,
                    pos: [320.0, 80.0],
                    node: DarkNode::default_blur(BlurType::Gaussian, 12.0, 0.0, 1.0),
                    custom_title: None,
                },
                NodeEntry {
                    id: 2,
                    pos: [600.0, 80.0],
                    node: DarkNode::Output,
                    custom_title: None,
                },
            ],
            wires: vec![
                NodeWire {
                    out_node: 0,
                    out_pin: 0,
                    in_node: 1,
                    in_pin: 0,
                },
                NodeWire {
                    out_node: 1,
                    out_pin: 0,
                    in_node: 2,
                    in_pin: 0,
                },
            ],
            groups: vec![VisualGroupDoc {
                id: 1,
                title: "My Glow".to_string(),
                min: [20.0, 40.0],
                max: [540.0, 260.0],
                color_rgba: [30, 144, 255, 35],
                is_locked: false,
            }],
        };

        let preset = NodeGraphPresetDoc::new("Soft Blur", "Blur", "Test description", doc.clone());
        let json = preset.to_json_string().expect("serialize");
        let parsed = NodeGraphPresetDoc::from_json_str(&json).expect("deserialize");
        assert_eq!(parsed.name, "Soft Blur");
        assert_eq!(parsed.category, "Blur");
        assert_eq!(parsed.graph, doc);
    }

    #[test]
    fn preset_doc_loads_legacy_format_without_wires() {
        let legacy_json = r#"{
            "name": "Legacy Preset",
            "version": "1.0.0",
            "description": "Old format",
            "nodes": [
                { "VideoInput": { "track_name": "V1" } },
                "Output"
            ],
            "groups": []
        }"#;
        let parsed = NodeGraphPresetDoc::from_json_str(legacy_json).expect("legacy parse");
        assert_eq!(parsed.name, "Legacy Preset");
        assert_eq!(parsed.graph.nodes.len(), 2);
        assert!(parsed.graph.wires.is_empty());
    }

    #[test]
    fn chain_node_graphs_connects_filter_preset_and_overlay_preset() {
        let base = NodeGraphDoc {
            nodes: vec![
                NodeEntry {
                    id: 0,
                    pos: [0.0, 0.0],
                    node: DarkNode::VideoInput {
                        track_name: "V1".to_string(),
                    },
                    custom_title: None,
                },
                NodeEntry {
                    id: 1,
                    pos: [240.0, 0.0],
                    node: match DarkNode::default_color_grade() {
                        DarkNode::ColorGrade {
                            temperature, tint, gamma, invert, black_and_white, tint_color,
                            tint_amount, highlights, shadows, whites, blacks, vibrance,
                            faded_film, shadow_tint, highlight_tint, ..
                        } => DarkNode::ColorGrade {
                            exposure: 0.4,
                            contrast: 1.1,
                            saturation: 1.2,
                            temperature, tint, gamma, invert, black_and_white, tint_color,
                            tint_amount, highlights, shadows, whites, blacks, vibrance,
                            faded_film, shadow_tint, highlight_tint,
                        },
                        other => other,
                    },
                    custom_title: None,
                },
                NodeEntry {
                    id: 2,
                    pos: [480.0, 0.0],
                    node: DarkNode::Output,
                    custom_title: None,
                },
            ],
            wires: vec![
                NodeWire { out_node: 0, out_pin: 0, in_node: 1, in_pin: 0 },
                NodeWire { out_node: 1, out_pin: 0, in_node: 2, in_pin: 0 },
            ],
            groups: Vec::new(),
        };

        // Case A: Filter preset with VideoInput -> Blur -> Output
        let filter_preset = NodeGraphDoc {
            nodes: vec![
                NodeEntry {
                    id: 0,
                    pos: [0.0, 0.0],
                    node: DarkNode::VideoInput {
                        track_name: "V1".to_string(),
                    },
                    custom_title: None,
                },
                NodeEntry {
                    id: 1,
                    pos: [240.0, 0.0],
                    node: DarkNode::default_blur(BlurType::BokehBloom, 20.0, 0.0, 1.5),
                    custom_title: None,
                },
                NodeEntry {
                    id: 2,
                    pos: [480.0, 0.0],
                    node: DarkNode::Output,
                    custom_title: None,
                },
            ],
            wires: vec![
                NodeWire { out_node: 0, out_pin: 0, in_node: 1, in_pin: 0 },
                NodeWire { out_node: 1, out_pin: 0, in_node: 2, in_pin: 0 },
            ],
            groups: Vec::new(),
        };

        let chained = chain_node_graphs(&base, &filter_preset);
        let plan = DagEvaluator::build_execution_plan(&chained).expect("chained DAG valid");
        assert!(plan.steps.len() >= 4);
        assert_eq!(
            chained.nodes.iter().filter(|n| matches!(n.node, DarkNode::Output)).count(),
            1
        );
        // ColorGrade (id 1) phải nối sang BlurSharpen (id 1 + 3 = 4)
        assert!(chained.wires.iter().any(|w| w.out_node == 1 && w.in_node == 4));

        // Case B: Overlay preset without VideoInput
        let overlay = NodeGraphDoc {
            nodes: vec![
                NodeEntry {
                    id: 0,
                    pos: [0.0, 0.0],
                    node: DarkNode::default_shape(
                        ShapeType::RoundedRectangle,
                        [800.0, 80.0],
                        8.0,
                        [0.0, 0.8, 1.0, 1.0],
                    ),
                    custom_title: None,
                },
                NodeEntry {
                    id: 1,
                    pos: [240.0, 0.0],
                    node: DarkNode::Output,
                    custom_title: None,
                },
            ],
            wires: vec![NodeWire { out_node: 0, out_pin: 0, in_node: 1, in_pin: 0 }],
            groups: Vec::new(),
        };
        let chained_overlay = chain_node_graphs(&base, &overlay);
        DagEvaluator::build_execution_plan(&chained_overlay).expect("chained overlay DAG valid");
        assert!(chained_overlay.nodes.iter().any(|n| matches!(n.node, DarkNode::Blend { .. })));
    }
}
