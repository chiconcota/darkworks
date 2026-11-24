use darkworks_node_graph::{DarkNode, GraphHistory, GraphSnapshot};
use egui::Pos2;
use std::collections::HashMap;

#[test]
fn test_graph_history_undo_redo() {
    let mut history = GraphHistory::new(10);
    let mut snarl = egui_snarl::Snarl::new();
    let groups = Vec::new();
    let mut titles = HashMap::new();

    // Snapshot 0: Rỗng
    let snap0 = GraphSnapshot {
        snarl: snarl.clone(),
        groups: groups.clone(),
        custom_titles: titles.clone(),
    };
    history.push_snapshot(snap0);

    // Chỉnh sửa 1: Thêm 1 node
    let n1 = snarl.insert_node(Pos2::new(10.0, 10.0), DarkNode::Output);
    titles.insert(n1, "Main Out".to_string());
    let snap1 = GraphSnapshot {
        snarl: snarl.clone(),
        groups: groups.clone(),
        custom_titles: titles.clone(),
    };
    history.push_snapshot(snap1);

    // Chỉnh sửa 2: Xóa node
    snarl.remove_node(n1);
    titles.remove(&n1);
    let snap2 = GraphSnapshot {
        snarl: snarl.clone(),
        groups: groups.clone(),
        custom_titles: titles.clone(),
    };

    assert!(history.can_undo());
    assert!(!history.can_redo());

    // Hoàn tác (Undo): Phải khôi phục lại Node 1
    let restored1 = history.undo(snap2).expect("Must undo");
    assert!(restored1.custom_titles.contains_key(&n1));
    assert!(history.can_redo());

    // Làm lại (Redo): Node 1 lại bị xóa
    let restored2 = history.redo(restored1).expect("Must redo");
    assert!(!restored2.custom_titles.contains_key(&n1));
}
