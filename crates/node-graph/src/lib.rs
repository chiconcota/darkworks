//! # Darkworks Node Graph (`darkworks-node-graph`)
//!
//! Lõi xử lý đồ thị có hướng không chu trình (DAG) cho Darkworks NLE Video Editor.
//! Tích hợp `egui_snarl` và các tiện ích tổ chức canvas ComfyUI Style.

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod compound;
pub mod custom_shader;
pub mod doc;
pub mod evaluator;
pub mod group;
pub mod history;
pub mod node;
pub mod node_controls;
mod node_controls_fx;
pub mod preset;
pub mod types;
pub mod viewer;

pub use compound::{BlackNodeSubgraph, ExposedPin};
pub use custom_shader::{WgslError, WgslParser, WgslUniform};
pub use darkworks_node_graph_core::{
    NodeEntry, NodeGraphDoc, NodeGraphPresetDoc, NodeWire, PresetDocError, VisualGroupDoc,
    chain_node_graphs, ensure_preset_extension, sanitize_preset_filename,
};
pub use doc::{doc_from_snarl, snarl_from_doc};
pub use evaluator::{DagError, DagEvaluator, DagExecutionPlan, DagStep, EvaluationContext};
pub use group::VisualGroup;
pub use history::{GraphHistory, GraphSnapshot};
pub use node::{
    ActionScope, AudioBand, BlendMode, BlurDimensions, BlurType, DarkNode, DistortType,
    KeyerMode, KeyerOutput, ShapeType, TextAlign, TimeMode, TransitionDirection, TransitionType,
};
pub use preset::{BuiltinPresetKind, NodeGraphPreset, PresetError};
pub use types::{NodeDataType, pin_color32};
pub use viewer::{DarkworksViewer, ZoomAction};
