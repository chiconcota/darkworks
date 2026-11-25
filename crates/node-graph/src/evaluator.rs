//! Bộ duyệt đồ thị có hướng không chu trình (DAG Evaluator) gắn với `egui_snarl::Snarl`.
//!
//! Ủy quyền thuật toán Kahn & tính toán tín hiệu sang `darkworks_node_graph_core::DagEvaluator`.

use std::collections::HashMap;
use egui_snarl::{NodeId, Snarl};

pub use darkworks_node_graph_core::{DagError, EvaluationContext};

use crate::doc::doc_from_snarl;
use crate::node::DarkNode;

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

/// Bộ đánh giá và thực thi đồ thị DAG cho `Snarl<DarkNode>`.
pub struct DagEvaluator;

impl DagEvaluator {
    /// Xây dựng kế hoạch thực thi bằng thuật toán Topological Sort (Kahn's Algorithm).
    pub fn build_execution_plan(snarl: &Snarl<DarkNode>) -> Result<DagExecutionPlan, DagError> {
        let doc = doc_from_snarl(snarl, &[], &HashMap::new());
        let doc_plan = darkworks_node_graph_core::DagEvaluator::build_execution_plan(&doc)?;

        let steps = doc_plan
            .steps
            .into_iter()
            .map(|s| DagStep {
                node_id: NodeId(s.node_id),
                node: s.node,
                dependencies: s.dependencies.into_iter().map(NodeId).collect(),
            })
            .collect();

        let execution_levels = doc_plan
            .execution_levels
            .into_iter()
            .map(|lvl| lvl.into_iter().map(NodeId).collect())
            .collect();

        Ok(DagExecutionPlan {
            steps,
            execution_levels,
        })
    }

    /// Tính toán luồng tín hiệu số (Scalar, Vector, Time, Audio) theo thứ tự Topo.
    #[must_use]
    pub fn evaluate_signals(
        plan: &DagExecutionPlan,
        timeline_time_sec: f64,
        audio_rms_peak: f32,
    ) -> EvaluationContext {
        let doc_plan = darkworks_node_graph_core::DagDocPlan {
            steps: plan
                .steps
                .iter()
                .map(|s| darkworks_node_graph_core::DagDocStep {
                    node_id: s.node_id.0,
                    node: s.node.clone(),
                    dependencies: s.dependencies.iter().map(|id| id.0).collect(),
                })
                .collect(),
            execution_levels: plan
                .execution_levels
                .iter()
                .map(|lvl| lvl.iter().map(|id| id.0).collect())
                .collect(),
        };
        darkworks_node_graph_core::DagEvaluator::evaluate_signals(
            &doc_plan,
            timeline_time_sec,
            audio_rms_peak,
        )
    }
}
