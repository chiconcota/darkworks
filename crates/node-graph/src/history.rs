//! Ngăn xếp quản lý lịch sử Hoàn tác / Làm lại (Undo / Redo Stack) cho Node Graph.

use std::collections::HashMap;
use egui_snarl::{NodeId, Snarl};
use crate::{DarkNode, VisualGroup};

/// Một ảnh chụp trạng thái đầy đủ của đồ thị DAG và các nhóm trực quan
#[derive(Clone, Debug)]
pub struct GraphSnapshot {
    pub snarl: Snarl<DarkNode>,
    pub groups: Vec<VisualGroup>,
    pub custom_titles: HashMap<NodeId, String>,
}

/// Trình quản lý ngăn xếp Hoàn tác / Làm lại (Undo / Redo Stack) cho Node Graph
#[derive(Clone, Debug)]
pub struct GraphHistory {
    undo_stack: Vec<GraphSnapshot>,
    redo_stack: Vec<GraphSnapshot>,
    max_depth: usize,
}

impl Default for GraphHistory {
    fn default() -> Self {
        Self::new(50)
    }
}

impl GraphHistory {
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth: if max_depth == 0 { 50 } else { max_depth },
        }
    }

    /// Lưu lại trạng thái hiện tại vào ngăn xếp Undo trước khi thực hiện chỉnh sửa
    pub fn push_snapshot(&mut self, snapshot: GraphSnapshot) {
        if self.max_depth == 0 {
            self.max_depth = 50;
        }
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(snapshot);
        self.redo_stack.clear();
    }

    /// Hoàn tác (Undo): Trả về trạng thái trước đó, đồng thời lưu trạng thái hiện tại vào Redo
    pub fn undo(&mut self, current: GraphSnapshot) -> Option<GraphSnapshot> {
        let prev = self.undo_stack.pop()?;
        self.redo_stack.push(current);
        Some(prev)
    }

    /// Làm lại (Redo): Trả về trạng thái kế tiếp, đồng thời lưu trạng thái hiện tại vào Undo
    pub fn redo(&mut self, current: GraphSnapshot) -> Option<GraphSnapshot> {
        let next = self.redo_stack.pop()?;
        self.undo_stack.push(current);
        Some(next)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}
