//! Bộ duyệt đồ thị có hướng không chu trình (DAG Evaluator) thuần dữ liệu.

use std::collections::{HashMap, VecDeque};
use thiserror::Error;

use crate::doc::NodeGraphDoc;
use crate::node::{DarkNode, TimeMode};
use crate::types::NodeDataType;

/// Các lỗi có thể phát sinh trong quá trình duyệt đồ thị DAG.
#[derive(Debug, Error, PartialEq)]
pub enum DagError {
    #[error("Phát hiện vòng lặp (Cycle) trong đồ thị: node {0:?}")]
    CycleDetected(Vec<usize>),

    #[error("Xung đột kiểu dữ liệu kết nối: nguồn {src_type:?} không tương thích đích {dst_type:?}")]
    TypeMismatch {
        src_type: NodeDataType,
        dst_type: NodeDataType,
    },

    #[error("Không tìm thấy Node đích trong đồ thị: {0:?}")]
    NodeNotFound(usize),
}

/// Một bước thực thi trong kế hoạch DAG thuần dữ liệu.
#[derive(Debug, Clone, PartialEq)]
pub struct DagDocStep {
    /// ID của node cần thực thi.
    pub node_id: usize,
    /// Loại node và tham số của nó.
    pub node: DarkNode,
    /// Danh sách các node phụ thuộc cần phải được tính toán trước.
    pub dependencies: Vec<usize>,
}

/// Kế hoạch thực thi đồ thị đã được sắp xếp topo.
#[derive(Debug, Clone, PartialEq)]
pub struct DagDocPlan {
    /// Danh sách các bước tuần tự theo thứ tự Topo.
    pub steps: Vec<DagDocStep>,
    /// Các tầng thực thi song song (Nodes ở cùng 1 tầng độc lập với nhau).
    pub execution_levels: Vec<Vec<usize>>,
}

/// Trạng thái tín hiệu đã được tính toán tại mỗi Node.
#[derive(Debug, Clone, Default)]
pub struct EvaluationContext {
    /// Thời gian Playhead hiện tại tính theo giây.
    pub timeline_time_sec: f64,
    /// Biên độ âm thanh Master RMS Peak hiện tại [0.0, 1.0].
    pub audio_rms_peak: f32,
    /// Bảng lưu giá trị số thực tính toán được tại mỗi Node (NodeId -> f32).
    pub scalar_outputs: HashMap<usize, f32>,
    /// Bảng lưu giá trị Vector 2D (NodeId -> [f32; 2]).
    pub vec2_outputs: HashMap<usize, [f32; 2]>,
    /// Bảng lưu màu sắc Color (NodeId -> [f32; 4]).
    pub color_outputs: HashMap<usize, [f32; 4]>,
}

/// Bộ đánh giá và thực thi đồ thị DAG thuần dữ liệu.
pub struct DagEvaluator;

impl DagEvaluator {
    /// Xây dựng kế hoạch thực thi từ [`NodeGraphDoc`] bằng Topological Sort (Kahn's Algorithm).
    pub fn build_execution_plan(doc: &NodeGraphDoc) -> Result<DagDocPlan, DagError> {
        let mut in_degree: HashMap<usize, usize> = HashMap::new();
        let mut adj_list: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut dependencies_map: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut all_nodes: HashMap<usize, &DarkNode> = HashMap::new();

        for entry in &doc.nodes {
            in_degree.insert(entry.id, 0);
            adj_list.insert(entry.id, Vec::new());
            dependencies_map.insert(entry.id, Vec::new());
            all_nodes.insert(entry.id, &entry.node);
        }

        for wire in &doc.wires {
            let src_idx = wire.out_node;
            let dst_idx = wire.in_node;

            if let (Some(src_node), Some(dst_node)) = (all_nodes.get(&src_idx), all_nodes.get(&dst_idx)) {
                let out_type = src_node.output_data_type(wire.out_pin);
                let in_type = dst_node.input_data_type(wire.in_pin);

                if !out_type.can_connect_to(in_type) {
                    return Err(DagError::TypeMismatch {
                        src_type: out_type,
                        dst_type: in_type,
                    });
                }
            }

            if let Some(deg) = in_degree.get_mut(&dst_idx) {
                *deg += 1;
            }
            if let Some(neighbors) = adj_list.get_mut(&src_idx) {
                neighbors.push(dst_idx);
            }
            if let Some(deps) = dependencies_map.get_mut(&dst_idx) {
                deps.push(src_idx);
            }
        }

        let mut queue: VecDeque<usize> = VecDeque::new();
        // Đảm bảo thứ tự ổn định theo ID cho các root nodes
        let mut roots: Vec<usize> = in_degree
            .iter()
            .filter_map(|(&id_idx, &deg)| (deg == 0).then_some(id_idx))
            .collect();
        roots.sort_unstable();
        queue.extend(roots);

        let mut steps = Vec::new();
        let mut execution_levels: Vec<Vec<usize>> = Vec::new();

        while !queue.is_empty() {
            let level_size = queue.len();
            let mut current_level = Vec::with_capacity(level_size);

            for _ in 0..level_size {
                if let Some(curr_idx) = queue.pop_front() {
                    if let Some(&node) = all_nodes.get(&curr_idx) {
                        current_level.push(curr_idx);
                        let deps = dependencies_map.get(&curr_idx).cloned().unwrap_or_default();
                        steps.push(DagDocStep {
                            node_id: curr_idx,
                            node: node.clone(),
                            dependencies: deps,
                        });
                    }

                    if let Some(neighbors) = adj_list.get(&curr_idx) {
                        for &neighbor_idx in neighbors {
                            if let Some(deg) = in_degree.get_mut(&neighbor_idx) {
                                *deg = deg.saturating_sub(1);
                                if *deg == 0 {
                                    queue.push_back(neighbor_idx);
                                }
                            }
                        }
                    }
                }
            }

            if !current_level.is_empty() {
                execution_levels.push(current_level);
            }
        }

        if steps.len() < all_nodes.len() {
            let mut unvisited: Vec<usize> = in_degree
                .into_iter()
                .filter(|(_, deg)| *deg > 0)
                .map(|(id, _)| id)
                .collect();
            unvisited.sort_unstable();
            return Err(DagError::CycleDetected(unvisited));
        }

        Ok(DagDocPlan {
            steps,
            execution_levels,
        })
    }

    /// Tính toán luồng tín hiệu số (Scalar, Vector, Time, Audio) theo thứ tự Topo.
    #[must_use]
    pub fn evaluate_signals(
        plan: &DagDocPlan,
        timeline_time_sec: f64,
        audio_rms_peak: f32,
    ) -> EvaluationContext {
        let empty = HashMap::new();
        Self::evaluate_signals_with_audio(plan, timeline_time_sec, audio_rms_peak, &empty)
    }

    /// Tính toán luồng tín hiệu số kèm bảng RMS riêng cho từng `AudioReactor` (`node_id -> rms`).
    #[must_use]
    pub fn evaluate_signals_with_audio(
        plan: &DagDocPlan,
        timeline_time_sec: f64,
        audio_rms_peak: f32,
        per_node_audio: &HashMap<usize, f32>,
    ) -> EvaluationContext {
        Self::evaluate_signals_for_clip(plan, timeline_time_sec, 0.0, audio_rms_peak, per_node_audio)
    }

    /// Tính toán luồng tín hiệu số theo ngữ cảnh Clip (`timeline_time_sec` & `clip_duration_sec`) và bảng RMS riêng.
    #[must_use]
    pub fn evaluate_signals_for_clip(
        plan: &DagDocPlan,
        timeline_time_sec: f64,
        clip_duration_sec: f64,
        audio_rms_peak: f32,
        per_node_audio: &HashMap<usize, f32>,
    ) -> EvaluationContext {
        let mut ctx = EvaluationContext {
            timeline_time_sec,
            audio_rms_peak,
            scalar_outputs: HashMap::new(),
            vec2_outputs: HashMap::new(),
            color_outputs: HashMap::new(),
        };

        for step in &plan.steps {
            let idx = step.node_id;
            match &step.node {
                DarkNode::TimeDriver {
                    speed,
                    mode,
                    amplitude,
                    offset,
                    vec2_axis,
                } => {
                    let raw_time = (timeline_time_sec as f32) * speed;
                    let base = match mode {
                        TimeMode::Linear => raw_time,
                        TimeMode::SineLoop => (raw_time.sin() + 1.0) * 0.5,
                        TimeMode::PingPong => {
                            let rem = raw_time.abs() % 2.0;
                            if rem < 1.0 { rem } else { 2.0 - rem }
                        }
                        TimeMode::Clip01 => {
                            if clip_duration_sec > 1e-6 {
                                (timeline_time_sec / clip_duration_sec).clamp(0.0, 1.0) as f32
                            } else {
                                0.0
                            }
                        }
                    };
                    let scalar = base * amplitude + offset;
                    ctx.scalar_outputs.insert(idx, scalar);
                    ctx.vec2_outputs
                        .insert(idx, [scalar * vec2_axis[0], scalar * vec2_axis[1]]);
                }
                DarkNode::AudioReactor {
                    multiplier,
                    vec2_axis,
                    ..
                } => {
                    let rms = per_node_audio.get(&idx).copied().unwrap_or(audio_rms_peak);
                    let val = (rms * multiplier).clamp(0.0, 10.0);
                    ctx.scalar_outputs.insert(idx, val);
                    ctx.vec2_outputs
                        .insert(idx, [val * vec2_axis[0], val * vec2_axis[1]]);
                }
                DarkNode::ShapeGenerator { color, size, .. } => {
                    ctx.color_outputs.insert(idx, *color);
                    ctx.vec2_outputs.insert(idx, *size);
                }
                DarkNode::TextGenerator { color, .. } => {
                    ctx.color_outputs.insert(idx, *color);
                }
                DarkNode::Transform2D { position, scale, .. } => {
                    let mut final_pos = *position;
                    let mut final_scale = *scale;

                    for &dep in &step.dependencies {
                        if let Some(&val) = ctx.scalar_outputs.get(&dep) {
                            final_scale[0] *= val.max(0.01);
                            final_scale[1] *= val.max(0.01);
                        }
                        if let Some(&pos_mod) = ctx.vec2_outputs.get(&dep) {
                            final_pos[0] += pos_mod[0];
                            final_pos[1] += pos_mod[1];
                        }
                    }

                    ctx.vec2_outputs.insert(idx, final_pos);
                    ctx.scalar_outputs.insert(idx, final_scale[0]);
                }
                DarkNode::DistortLens { intensity, center, .. } => {
                    let mut final_intensity = *intensity;
                    let mut final_center = *center;

                    for &dep in &step.dependencies {
                        if let Some(&val) = ctx.scalar_outputs.get(&dep) {
                            final_intensity *= val;
                        }
                        if let Some(&center_mod) = ctx.vec2_outputs.get(&dep) {
                            final_center[0] += center_mod[0];
                            final_center[1] += center_mod[1];
                        }
                    }

                    ctx.scalar_outputs.insert(idx, final_intensity);
                    ctx.vec2_outputs.insert(idx, final_center);
                }
                DarkNode::Reroute { .. } => {
                    if let Some(&dep) = step.dependencies.first() {
                        if let Some(&s) = ctx.scalar_outputs.get(&dep) {
                            ctx.scalar_outputs.insert(idx, s);
                        }
                        if let Some(&v) = ctx.vec2_outputs.get(&dep) {
                            ctx.vec2_outputs.insert(idx, v);
                        }
                        if let Some(&c) = ctx.color_outputs.get(&dep) {
                            ctx.color_outputs.insert(idx, c);
                        }
                    }
                }
                _ => {}
            }
        }

        ctx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::{NodeEntry, NodeGraphDoc, NodeWire};
    use crate::node_fx_types::TransitionType;

    #[test]
    fn time_driver_clip01_normalizes_by_clip_duration_and_clamps() {
        let doc = NodeGraphDoc {
            nodes: vec![
                NodeEntry {
                    id: 0,
                    pos: [0.0, 0.0],
                    node: DarkNode::default_time_driver(0.5, TimeMode::Clip01),
                    custom_title: None,
                },
                NodeEntry {
                    id: 1,
                    pos: [200.0, 0.0],
                    node: DarkNode::default_transition(TransitionType::CrossDissolve, 0.0, 0.1),
                    custom_title: None,
                },
            ],
            wires: vec![NodeWire {
                out_node: 0,
                out_pin: 0,
                in_node: 1,
                in_pin: 2,
            }],
            groups: Vec::new(),
        };
        let plan = DagEvaluator::build_execution_plan(&doc).expect("valid plan");
        let empty = HashMap::new();

        let start = DagEvaluator::evaluate_signals_for_clip(&plan, 0.0, 4.0, 0.0, &empty);
        let mid = DagEvaluator::evaluate_signals_for_clip(&plan, 2.0, 4.0, 0.0, &empty);
        let end = DagEvaluator::evaluate_signals_for_clip(&plan, 4.0, 4.0, 0.0, &empty);
        let over = DagEvaluator::evaluate_signals_for_clip(&plan, 6.0, 4.0, 0.0, &empty);

        assert!((start.scalar_outputs[&0] - 0.0).abs() < 1e-6);
        assert!((mid.scalar_outputs[&0] - 0.5).abs() < 1e-6);
        assert!((end.scalar_outputs[&0] - 1.0).abs() < 1e-6);
        assert!((over.scalar_outputs[&0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn overlay_hint_reads_linear_duration_through_reroute_and_tracks() {
        let doc = NodeGraphDoc {
            nodes: vec![
                NodeEntry {
                    id: 0,
                    pos: [0.0, 0.0],
                    node: DarkNode::VideoInput { track_name: "V1".into() },
                    custom_title: None,
                },
                NodeEntry {
                    id: 1,
                    pos: [0.0, 100.0],
                    node: DarkNode::VideoInput { track_name: "V2".into() },
                    custom_title: None,
                },
                NodeEntry {
                    id: 2,
                    pos: [0.0, 200.0],
                    node: DarkNode::default_time_driver(0.5, TimeMode::Linear),
                    custom_title: None,
                },
                NodeEntry {
                    id: 3,
                    pos: [120.0, 200.0],
                    node: DarkNode::Reroute {
                        data_type: NodeDataType::Scalar,
                    },
                    custom_title: None,
                },
                NodeEntry {
                    id: 4,
                    pos: [300.0, 100.0],
                    node: DarkNode::default_transition(TransitionType::CrossDissolve, 0.5, 0.1),
                    custom_title: None,
                },
            ],
            wires: vec![
                NodeWire { out_node: 0, out_pin: 0, in_node: 4, in_pin: 0 },
                NodeWire { out_node: 1, out_pin: 0, in_node: 4, in_pin: 1 },
                NodeWire { out_node: 2, out_pin: 0, in_node: 3, in_pin: 0 },
                NodeWire { out_node: 3, out_pin: 0, in_node: 4, in_pin: 2 },
            ],
            groups: Vec::new(),
        };

        let hint = doc.overlay_hint();
        assert!(hint.has_transition);
        assert!(!hint.has_clip01_driver);
        assert_eq!(hint.transition_tracks, vec!["V1".to_string(), "V2".to_string()]);
        assert!(hint.driver_duration_sec.is_some_and(|d| (d - 2.0).abs() < 1e-4));
    }
}
