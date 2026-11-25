//! Thanh menu Thêm Node và Tải Preset cho Node Graph Panel.

use darkworks_node_graph::{
    ActionScope, AudioBand, BlendMode, BlurType, BuiltinPresetKind, DarkNode, DistortType,
    NodeDataType, NodeGraphPreset, ShapeType, TimeMode, TransitionType,
};
use egui::{Pos2, Ui};

use super::node_graph::NodeGraphState;

/// Hiển thị Menu Thêm 18 Atomic Nodes chuẩn Nuke / Fusion
pub fn render_add_node_menu(ui: &mut Ui, state: &mut NodeGraphState) {
    ui.menu_button("➕ Add Node", |ui| {
        ui.label(egui::RichText::new("--- 1. NGUỒN PHÁT (GENERATORS) ---").weak());
        if ui.button("🎬 Video Input").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(100.0, 100.0), DarkNode::VideoInput { track_name: "V1".to_string() });
            ui.close();
        }
        if ui.button("🔤 Text Generator").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(100.0, 100.0), DarkNode::TextGenerator {
                text: "Tiêu đề mới".to_string(),
                font_size: 32.0,
                color: [1.0, 1.0, 1.0, 1.0],
                tracking: 0.0,
                line_spacing: 1.0,
            });
            ui.close();
        }
        if ui.button("🟩 Shape / Solid Matte").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(100.0, 100.0), DarkNode::ShapeGenerator {
                shape_type: ShapeType::RoundedRectangle,
                size: [400.0, 100.0],
                corner_radius: 10.0,
                color: [0.0, 0.8, 1.0, 1.0],
            });
            ui.close();
        }
        if ui.button("⏱️ Time Driver").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(100.0, 100.0), DarkNode::TimeDriver { speed: 1.0, mode: TimeMode::Linear });
            ui.close();
        }

        ui.separator();
        ui.label(egui::RichText::new("--- 2. CHỈNH MÀU & QUANG HỌC ---").weak());
        if ui.button("🎨 Color Grade (Invert/B&W)").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(200.0, 100.0), DarkNode::ColorGrade {
                exposure: 0.0,
                contrast: 1.0,
                saturation: 1.0,
                temperature: 0.0,
                tint: 0.0,
                gamma: 1.0,
                invert: false,
                black_and_white: false,
                tint_color: [1.0, 1.0, 1.0, 1.0],
                tint_amount: 0.0,
            });
            ui.close();
        }
        if ui.button("💧 Blur / Sharpen").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(200.0, 100.0), DarkNode::BlurSharpen {
                blur_type: BlurType::Gaussian,
                radius: 10.0,
                angle_deg: 0.0,
                highlight_boost: 1.0,
            });
            ui.close();
        }
        if ui.button("🌀 Distort / Lens").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(200.0, 100.0), DarkNode::DistortLens {
                distort_type: DistortType::RippleWave,
                intensity: 0.5,
                radius: 1.0,
                center: [0.0, 0.0],
            });
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
            state.snarl.insert_node(Pos2::new(200.0, 100.0), DarkNode::Transition {
                transition_type: TransitionType::CrossDissolve,
                progress: 0.5,
                feather: 0.1,
            });
            ui.close();
        }

        ui.separator();
        ui.label(egui::RichText::new("--- 5. TÍN HIỆU & TỰ ĐỘNG HÓA ---").weak());
        if ui.button("🔊 Audio Reactor").clicked() {
            state.push_undo();
            state.snarl.insert_node(Pos2::new(200.0, 100.0), DarkNode::AudioReactor {
                band: AudioBand::Bass,
                multiplier: 1.5,
                smooth: 0.2,
            });
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

fn load_preset_into_scratchpad(state: &mut NodeGraphState, kind: BuiltinPresetKind, label: &str) {
    state.push_undo();
    let (snarl, groups) = NodeGraphPreset::load_builtin(kind);
    state.snarl = snarl;
    state.groups = groups;
    state.custom_titles.clear();
    state.selected_group = None;
    state.last_hovered_group = None;
    state.detach_to_scratchpad();
    state.status_message = format!(
        "Đã nạp: {label} (Nháp — bấm 'Add as FX Clip' để tạo mới hoặc 'Apply to Clip' để gắn clip đang chọn)"
    );
}

/// Hiển thị Menu nạp 5 Presets mẫu chuẩn
pub fn render_load_preset_menu(ui: &mut Ui, state: &mut NodeGraphState) {
    ui.menu_button("📂 Load Preset", |ui| {
        const PRESETS: &[(&str, BuiltinPresetKind, &str)] = &[
            ("1. 📰 News Ticker Banner", BuiltinPresetKind::NewsTickerBanner, "1. News Ticker Banner"),
            ("2. 🌊 Audio Waveform Line (60 FPS)", BuiltinPresetKind::ContinuousAudioWaveform, "2. Continuous Audio Waveform Line"),
            ("3. 🎯 Spotlight Invert", BuiltinPresetKind::SpotlightInvert, "3. Spotlight Invert"),
            ("4. ⚡ Audio Beat Invert Flash", BuiltinPresetKind::AudioBeatFlash, "4. Audio Beat Invert Flash"),
            ("5. ✨ Cinematic Soft Glow", BuiltinPresetKind::CinematicSoftGlow, "5. Cinematic Soft Glow"),
        ];
        for &(btn_text, kind, label) in PRESETS {
            if ui.button(btn_text).clicked() {
                load_preset_into_scratchpad(state, kind, label);
                ui.close();
            }
        }
    });
}
