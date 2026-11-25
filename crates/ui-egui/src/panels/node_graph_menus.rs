//! Thanh menu Thêm Node và Tải Preset cho Node Graph Panel.

use darkworks_node_graph::{
    ActionScope, AudioBand, BlendMode, BlurType, DarkNode, DistortType, NodeDataType, ShapeType,
    TimeMode, TransitionType,
};
use egui::{Pos2, Ui};

use super::node_graph::NodeGraphState;

/// Hiển thị Menu Thêm 18 Atomic Nodes chuẩn Nuke / Fusion
pub fn render_add_node_menu(ui: &mut Ui, state: &mut NodeGraphState) {
    ui.menu_button("➕ Add Node", |ui| {
        ui.label(egui::RichText::new("--- 1. NGUỒN PHÁT (GENERATORS) ---").weak());
        if ui.button("🎬 Video Input").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(100.0, 100.0), DarkNode::VideoInput { track_name: "Auto".to_string() });
            ui.close();
        }
        if ui.button("🔤 Text Generator").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(100.0, 100.0), DarkNode::default_text("Tiêu đề mới"));
            ui.close();
        }
        if ui.button("🟩 Shape / Solid Matte").clicked() {
            state.push_undo();
            state.snarl.insert_node(
                Pos2::new(100.0, 100.0),
                DarkNode::default_shape(ShapeType::RoundedRectangle, [400.0, 100.0], 10.0, [0.0, 0.8, 1.0, 1.0]),
            );
            ui.close();
        }
        if ui.button("⏱️ Time Driver").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(100.0, 100.0), DarkNode::default_time_driver(1.0, TimeMode::Linear));
            ui.close();
        }

        ui.separator();
        ui.label(egui::RichText::new("--- 2. CHỈNH MÀU & QUANG HỌC ---").weak());
        if ui.button("🎨 Color Grade (Lumetri/Invert/B&W)").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(200.0, 100.0), DarkNode::default_color_grade());
            ui.close();
        }
        if ui.button("💧 Blur / Sharpen").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(200.0, 100.0), DarkNode::default_blur(BlurType::Gaussian, 10.0, 0.0, 1.0));
            ui.close();
        }
        if ui.button("🌀 Distort / Lens / Stylize").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(200.0, 100.0), DarkNode::default_distort(DistortType::RippleWave, 0.5, 1.0, [0.0, 0.0]));
            ui.close();
        }
        if ui.button("🟢 Ultra Keyer").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(200.0, 100.0), DarkNode::Keyer {
                key_color: [0.0, 1.0, 0.0],
                similarity: 0.4,
                smoothness: 0.1,
                spill_suppress: 0.5,
            });
            ui.close();
        }

        ui.separator();
        ui.label(egui::RichText::new("--- 3. BIẾN HÌNH & HÒA TRỘN ---").weak());
        if ui.button("📐 Transform 2D (Flip H/V)").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(200.0, 100.0), DarkNode::Transform2D {
                position: [0.0, 0.0],
                scale: [1.0, 1.0],
                rotation_deg: 0.0,
                anchor: [0.5, 0.5],
                opacity: 1.0,
                flip_h: false,
                flip_v: false,
            });
            ui.close();
        }
        if ui.button("✂️ Crop / Mask").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(200.0, 100.0), DarkNode::CropMask {
                top: 0.0,
                bottom: 0.0,
                left: 0.0,
                right: 0.0,
                feather: 0.0,
            });
            ui.close();
        }
        if ui.button("🎭 Blend / Mix (26 Modes)").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(200.0, 100.0), DarkNode::Blend {
                mode: BlendMode::Normal,
                opacity: 1.0,
            });
            ui.close();
        }

        ui.separator();
        ui.label(egui::RichText::new("--- 4. CHUYỂN CẢNH ---").weak());
        if ui.button("⚡ Transition").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(200.0, 100.0), DarkNode::default_transition(TransitionType::CrossDissolve, 0.5, 0.1));
            ui.close();
        }

        ui.separator();
        ui.label(egui::RichText::new("--- 5. TÍN HIỆU & TỰ ĐỘNG HÓA ---").weak());
        if ui.button("🔊 Audio Reactor").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(200.0, 100.0), DarkNode::default_audio_reactor(AudioBand::Bass, 1.5, 0.2));
            ui.close();
        }
        if ui.button("⚡ Action Node").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(250.0, 100.0), DarkNode::ActionNode {
                name: "Batch Ripple Cut".to_string(),
                scope: ActionScope::MarkedInOut,
                is_running: false,
            });
            ui.close();
        }

        ui.separator();
        ui.label(egui::RichText::new("--- 6. MỞ RỘNG & COMFYUI ---").weak());
        if ui.button("⬛ Black Box (Compound)").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(250.0, 100.0), DarkNode::BlackNode {
                name: "Custom FX Group".to_string(),
                subgraph_id: "fx_group_1".to_string(),
                exposed_pins: vec!["Image In".to_string()],
            });
            ui.close();
        }
        if ui.button("💻 Custom WGSL Shader").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(250.0, 100.0), DarkNode::CustomWgsl {
                name: "Custom Filter".to_string(),
                code: "// WGSL Shader Compute\noutput_color = vec4<f32>(1.0 - in.r, 1.0 - in.g, 1.0 - in.b, in.a);".to_string(),
            });
            ui.close();
        }
        if ui.button("📝 Sticky Note").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(100.0, 200.0), DarkNode::StickyNote {
                title: "Ghi chú".to_string(),
                content: "Nội dung ghi chú...".to_string(),
                color_idx: 0,
            });
            ui.close();
        }
        if ui.button("● Reroute Dot").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(200.0, 200.0), DarkNode::Reroute { data_type: NodeDataType::Texture });
            ui.close();
        }
        if ui.button("🖥️ Final Output").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(300.0, 200.0), DarkNode::Output);
            ui.close();
        }
    });
}
