//! Bộ duyệt đồ thị có hướng không chu trình (DAG Evaluator).
//!
//! Triển khai thuật toán Topological Sort (Kahn's Algorithm), phát hiện vòng lặp vô tận (Cycle Detection),
//! và tính toán luồng tín hiệu (Signal Propagation) từ các Node nguồn tới Node kết xuất.

use std::collections::{HashMap, VecDeque};
use egui_snarl::{NodeId, Snarl};
use thiserror::Error;

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

/// Một bước thực thi trong kế hoạch DAG.
#[derive(Debug, Clone, PartialEq)]
pub struct DagStep {
    /// ID của node cần thực thi.
    pub node_id: NodeId,
    /// Loại node và tham số của nó.
    pub node: DarkNode,
    /// Danh sách các node phụ thuộc cần phải được tính toán trước.
    pub dependencies: Vec<NodeId>,
}

/// Kế hoạch thực thi đồ thị đã được sắp xếp topo.
#[derive(Debug, Clone, PartialEq)]
pub struct DagExecutionPlan {
    /// Danh sách các bước tuần tự theo thứ tự Topo.
    pub steps: Vec<DagStep>,
    /// Các tầng thực thi song song (Nodes ở cùng 1 tầng độc lập với nhau).
    pub execution_levels: Vec<Vec<NodeId>>,
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

/// Bộ đánh giá và thực thi đồ thị DAG.
pub struct DagEvaluator;

impl DagEvaluator {
    /// Xây dựng kế hoạch thực thi bằng thuật toán Topological Sort (Kahn's Algorithm).
    ///
    /// Trả về `DagError::CycleDetected` nếu người dùng vô tình nối dây tạo thành vòng lặp.
    pub fn build_execution_plan(snarl: &Snarl<DarkNode>) -> Result<DagExecutionPlan, DagError> {
        let mut in_degree: HashMap<usize, usize> = HashMap::new();
        let mut adj_list: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut dependencies_map: HashMap<usize, Vec<NodeId>> = HashMap::new();
        let mut all_nodes: HashMap<usize, (NodeId, DarkNode)> = HashMap::new();

        // 1. Thu thập toàn bộ nodes hiện có
        for (node_id, node) in snarl.node_ids() {
            let id_idx = node_id.0;
            in_degree.insert(id_idx, 0);
            adj_list.insert(id_idx, Vec::new());
            dependencies_map.insert(id_idx, Vec::new());
            all_nodes.insert(id_idx, (node_id, node.clone()));
        }

        // 2. Thu thập các cạnh nối dây và xác thực kiểu dữ liệu
        for (out_pin, in_pin) in snarl.wires() {
            let src_idx = out_pin.node.0;
            let dst_idx = in_pin.node.0;

            // Kiểm tra tính tương thích của Socket
            if let (Some((_, src_node)), Some((_, dst_node))) = (all_nodes.get(&src_idx), all_nodes.get(&dst_idx)) {
                let out_type = src_node.output_data_type(out_pin.output);
                let in_type = dst_node.input_data_type(in_pin.input);

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
                deps.push(out_pin.node);
            }
        }

        // 3. Khởi tạo hàng đợi với các Node có in-degree = 0 (Roots / Generators)
        let mut queue: VecDeque<usize> = VecDeque::new();
        for (&id_idx, &deg) in &in_degree {
            if deg == 0 {
                queue.push_back(id_idx);
            }
        }

        let mut steps = Vec::new();
        let mut execution_levels: Vec<Vec<NodeId>> = Vec::new();

        // 4. Duyệt theo tầng (Kahn's algorithm)
        while !queue.is_empty() {
            let level_size = queue.len();
            let mut current_level = Vec::with_capacity(level_size);

            for _ in 0..level_size {
                if let Some(curr_idx) = queue.pop_front() {
                    if let Some((node_id, node)) = all_nodes.get(&curr_idx) {
                        current_level.push(*node_id);
                        let deps = dependencies_map.get(&curr_idx).cloned().unwrap_or_default();
                        steps.push(DagStep {
                            node_id: *node_id,
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

        // 5. Kiểm tra chu trình (Cycle Detection)
        if steps.len() < all_nodes.len() {
            let unvisited: Vec<usize> = in_degree
                .into_iter()
                .filter(|(_, deg)| *deg > 0)
                .map(|(id, _)| id)
                .collect();
            return Err(DagError::CycleDetected(unvisited));
        }

        Ok(DagExecutionPlan {
            steps,
            execution_levels,
        })
    }

    /// Tính toán luồng tín hiệu số (Scalar, Vector, Time, Audio) theo thứ tự Topo.
    pub fn evaluate_signals(
        plan: &DagExecutionPlan,
        timeline_time_sec: f64,
        audio_rms_peak: f32,
    ) -> EvaluationContext {
        let mut ctx = EvaluationContext {
            timeline_time_sec,
            audio_rms_peak,
            scalar_outputs: HashMap::new(),
            vec2_outputs: HashMap::new(),
            color_outputs: HashMap::new(),
        };

        for step in &plan.steps {
            let idx = step.node_id.0;
            match &step.node {
                DarkNode::TimeDriver { speed, mode } => {
                    let raw_time = (timeline_time_sec as f32) * speed;
                    let val = match mode {
                        TimeMode::Linear => raw_time,
                        TimeMode::SineLoop => (raw_time.sin() + 1.0) * 0.5,
                        TimeMode::PingPong => {
                            let rem = raw_time % 2.0;
                            if rem < 1.0 { rem } else { 2.0 - rem }
                        }
                    };
                    ctx.scalar_outputs.insert(idx, val);
                }
                DarkNode::AudioReactor { multiplier, smooth: _, band: _ } => {
                    let val = (audio_rms_peak * multiplier).clamp(0.0, 10.0);
                    ctx.scalar_outputs.insert(idx, val);
                }
                DarkNode::ShapeGenerator { color, size, .. } => {
                    ctx.color_outputs.insert(idx, *color);
                    ctx.vec2_outputs.insert(idx, *size);
                }
                DarkNode::TextGenerator { color, .. } => {
                    ctx.color_outputs.insert(idx, *color);
                }
                DarkNode::Transform2D { position, scale, .. } => {
                    // Nếu có node phụ thuộc xuất scalar hoặc vec2, kế thừa và cập nhật
                    let mut final_pos = *position;
                    let mut final_scale = *scale;

                    for &dep in &step.dependencies {
                        if let Some(&val) = ctx.scalar_outputs.get(&dep.0) {
                            // Audio bounce hoặc Time drive vào scale
                            final_scale[0] *= val.max(0.01);
                            final_scale[1] *= val.max(0.01);
                        }
                        if let Some(&pos_mod) = ctx.vec2_outputs.get(&dep.0) {
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
                        if let Some(&val) = ctx.scalar_outputs.get(&dep.0) {
                            final_intensity *= val;
                        }
                        if let Some(&center_mod) = ctx.vec2_outputs.get(&dep.0) {
                            final_center[0] += center_mod[0];
                            final_center[1] += center_mod[1];
                        }
                    }

                    ctx.scalar_outputs.insert(idx, final_intensity);
                    ctx.vec2_outputs.insert(idx, final_center);
                }
                DarkNode::Transition { progress, .. } => {
                    let mut final_prog = *progress;
                    for &dep in &step.dependencies {
                        if let Some(&val) = ctx.scalar_outputs.get(&dep.0) {
                            final_prog = val.clamp(0.0, 1.0);
                        }
                    }
                    ctx.scalar_outputs.insert(idx, final_prog);
                }
                _ => {}
            }
        }

        ctx
    }
}
