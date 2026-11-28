//! # Darkworks Node Graph Core (`darkworks-node-graph-core`)
//!
//! Lõi dữ liệu thuần (không phụ thuộc UI/`egui`) cho hệ thống đồ thị DAG của Darkworks.
//! Được chia sẻ an toàn giữa `crates/project` (L1), `crates/render` (L3), `crates/engine` (L4)
//! và `crates/node-graph` (L5).

#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod custom_shader;
pub mod doc;
pub mod evaluator;
pub mod node;
pub mod node_fx_types;
pub mod preset;
pub mod types;

pub use custom_shader::{WgslError, WgslParser, WgslUniform};
pub use doc::{NodeEntry, NodeGraphDoc, NodeWire, OverlayPlacementHint, VisualGroupDoc};
pub use evaluator::{DagDocPlan, DagDocStep, DagError, DagEvaluator, EvaluationContext};
pub use node::{
    ActionScope, AudioBand, BlendMode, BlurDimensions, BlurType, DarkNode, DistortType,
    KeyerMode, KeyerOutput, ShapeType, TextAlign, TimeMode, TransitionDirection, TransitionType,
};
pub use preset::{
    NodeGraphPresetDoc, PresetDocError, chain_node_graphs, ensure_preset_extension,
    sanitize_preset_filename,
};
pub use types::NodeDataType;
