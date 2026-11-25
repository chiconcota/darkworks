//! Chuyển đổi 2 chiều giữa đồ thị giao diện (`egui_snarl::Snarl<DarkNode>`, `VisualGroup`)
//! và mô hình dữ liệu thuần [`darkworks_node_graph_core::NodeGraphDoc`].

use std::collections::HashMap;
use darkworks_node_graph_core::{NodeEntry, NodeGraphDoc, NodeWire, VisualGroupDoc};
use egui::{Color32, Pos2, Vec2};
use egui_snarl::{InPinId, NodeId, OutPinId, Snarl};

use crate::group::VisualGroup;
use crate::node::DarkNode;

/// Chuyển đổi trạng thái `Snarl` + `VisualGroup` + `custom_titles` thành [`NodeGraphDoc`] để lưu xuống `project.json` và render.
#[must_use]
pub fn doc_from_snarl(
    snarl: &Snarl<DarkNode>,
    groups: &[VisualGroup],
    custom_titles: &HashMap<NodeId, String>,
) -> NodeGraphDoc {
    let mut nodes = Vec::new();
    for (node_id, pos, node) in snarl.nodes_pos_ids() {
        nodes.push(NodeEntry {
            id: node_id.0,
            pos: [pos.x, pos.y],
            node: node.clone(),
            custom_title: custom_titles.get(&node_id).cloned(),
        });
    }
    nodes.sort_by_key(|n| n.id);

    let mut wires: Vec<NodeWire> = snarl
        .wires()
        .map(|(out_pin, in_pin)| NodeWire {
            out_node: out_pin.node.0,
            out_pin: out_pin.output,
            in_node: in_pin.node.0,
            in_pin: in_pin.input,
        })
        .collect();
    wires.sort_by_key(|w| (w.in_node, w.in_pin, w.out_node, w.out_pin));

    let groups_doc = groups
        .iter()
        .map(|g| VisualGroupDoc {
            id: g.id,
            title: g.title.clone(),
            min: g.min,
            max: g.max,
            color_rgba: g.color_rgba,
            is_locked: g.is_locked,
        })
        .collect();

    NodeGraphDoc {
        nodes,
        wires,
        groups: groups_doc,
    }
}

/// Khôi phục `Snarl<DarkNode>`, `Vec<VisualGroup>` và `custom_titles` từ [`NodeGraphDoc`].
#[must_use]
pub fn snarl_from_doc(
    doc: &NodeGraphDoc,
) -> (Snarl<DarkNode>, Vec<VisualGroup>, HashMap<NodeId, String>) {
    let mut snarl = Snarl::<DarkNode>::new();
    let mut id_map: HashMap<usize, NodeId> = HashMap::new();
    let mut custom_titles = HashMap::new();

    for entry in &doc.nodes {
        let new_id = snarl.insert_node(Pos2::new(entry.pos[0], entry.pos[1]), entry.node.clone());
        id_map.insert(entry.id, new_id);
        if let Some(title) = &entry.custom_title {
            custom_titles.insert(new_id, title.clone());
        }
    }

    for wire in &doc.wires {
        if let (Some(&out_node), Some(&in_node)) = (id_map.get(&wire.out_node), id_map.get(&wire.in_node)) {
            snarl.connect(
                OutPinId {
                    node: out_node,
                    output: wire.out_pin,
                },
                InPinId {
                    node: in_node,
                    input: wire.in_pin,
                },
            );
        }
    }

    let groups = doc
        .groups
        .iter()
        .map(|g| {
            let mut vg = VisualGroup::new(
                g.id,
                g.title.clone(),
                Pos2::new(g.min[0], g.min[1]),
                Vec2::new(g.max[0] - g.min[0], g.max[1] - g.min[1]),
                Color32::from_rgba_unmultiplied(g.color_rgba[0], g.color_rgba[1], g.color_rgba[2], g.color_rgba[3]),
            );
            vg.is_locked = g.is_locked;
            vg
        })
        .collect();

    (snarl, groups, custom_titles)
}
