//! Bộ điều khiển giao diện UI Controls cho các Node trong Darkworks Node Graph.
//!
//! Tách biệt phần hiển thị thông số và điều khiển của 18 Atomic Nodes để giữ kiến trúc
//! tinh gọn, mô-đun hóa cao và tuân thủ tuyệt đối chuẩn trần dòng code (< 700 dòng).

use egui::{ComboBox, CursorIcon, DragValue, Response, Ui};
use crate::node::{
    BlurType, DarkNode, DistortType, ShapeType, TimeMode, TransitionType,
};

/// Helper gán con trỏ trượt hai chiều cho DragValue
#[inline]
pub fn drag_val(ui: &mut Ui, dv: DragValue<'_>) -> Response {
    let r = ui.add(dv);
    if r.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
    }
    r
}

/// Helper gán con trỏ I-beam cho ô nhập text
#[inline]
pub fn text_edit(ui: &mut Ui, text: &mut String) -> Response {
    let r = ui.text_edit_singleline(text);
    if r.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::Text);
    }
    r
}

/// Helper gán con trỏ I-beam cho ô soạn thảo nhiều dòng
#[inline]
pub fn text_edit_multi(ui: &mut Ui, text: &mut String) -> Response {
    let r = ui.text_edit_multiline(text);
    if r.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::Text);
    }
    r
}

/// Helper gán con trỏ ngón tay chỉ cho checkbox
#[inline]
pub fn check_box(ui: &mut Ui, checked: &mut bool, text: &str) -> Response {
    let r = ui.checkbox(checked, text);
    if r.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    r
}

/// Helper gán con trỏ ngón tay chỉ cho nút bấm
#[inline]
pub fn btn(ui: &mut Ui, text: &str) -> Response {
    let r = ui.button(text);
    if r.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    r
}

/// Render toàn bộ các thanh trượt, ô nhập dữ liệu và dropdown của 18 Atomic Nodes.
pub fn render_node_body(node: &mut DarkNode, ui: &mut Ui) {
    match node {
        DarkNode::TextGenerator { text, font_size, tracking, line_spacing, .. } => {
            ui.horizontal(|ui| {
                ui.label("Text:");
                text_edit(ui, text);
            });
            ui.horizontal(|ui| {
                ui.label("Size:");
                drag_val(ui, DragValue::new(font_size).range(8.0..=200.0));
                ui.label("Track:");
                drag_val(ui, DragValue::new(tracking).range(-10.0..=50.0).speed(0.1));
            });
            ui.horizontal(|ui| {
                ui.label("Line Spacing:");
                drag_val(ui, DragValue::new(line_spacing).range(0.5..=3.0).speed(0.05));
            });
        }
        DarkNode::ShapeGenerator { shape_type, size, corner_radius, .. } => {
            ui.horizontal(|ui| {
                ui.label("Width:");
                drag_val(ui, DragValue::new(&mut size[0]).range(1.0..=3840.0));
                ui.label("Height:");
                drag_val(ui, DragValue::new(&mut size[1]).range(1.0..=2160.0));
            });
            if *shape_type == ShapeType::RoundedRectangle {
                ui.horizontal(|ui| {
                    ui.label("Radius:");
                    drag_val(ui, DragValue::new(corner_radius).range(0.0..=100.0));
                });
            }
        }
        DarkNode::TimeDriver { speed, mode } => {
            ui.horizontal(|ui| {
                ui.label("Speed:");
                drag_val(ui, DragValue::new(speed).speed(0.1));
            });
            ComboBox::from_label("Mode")
                .selected_text(format!("{mode:?}"))
                .show_ui(ui, |ui| {
                    ui.selectable_value(mode, TimeMode::Linear, "Linear");
                    ui.selectable_value(mode, TimeMode::SineLoop, "SineLoop");
                    ui.selectable_value(mode, TimeMode::PingPong, "PingPong");
                });
        }
        DarkNode::ColorGrade {
            exposure, contrast, saturation, temperature, tint, gamma, invert, black_and_white,
        } => {
            ui.horizontal(|ui| {
                ui.label("Exp:");
                drag_val(ui, DragValue::new(exposure).range(-5.0..=5.0).speed(0.05));
                ui.label("Contrast:");
                drag_val(ui, DragValue::new(contrast).range(0.0..=3.0).speed(0.05));
            });
            ui.horizontal(|ui| {
                ui.label("Sat:");
                drag_val(ui, DragValue::new(saturation).range(0.0..=3.0).speed(0.05));
                ui.label("Temp:");
                drag_val(ui, DragValue::new(temperature).range(-100.0..=100.0).speed(0.5));
            });
            ui.horizontal(|ui| {
                ui.label("Tint:");
                drag_val(ui, DragValue::new(tint).range(-100.0..=100.0).speed(0.5));
                ui.label("Gamma:");
                drag_val(ui, DragValue::new(gamma).range(0.1..=4.0).speed(0.05));
            });
            ui.horizontal(|ui| {
                check_box(ui, invert, "⚡ Invert Colors");
                check_box(ui, black_and_white, "⬛ B&W");
            });
        }
        DarkNode::BlurSharpen { blur_type, radius, angle_deg, highlight_boost } => {
            ComboBox::from_label("Type")
                .selected_text(format!("{blur_type:?}"))
                .show_ui(ui, |ui| {
                    ui.selectable_value(blur_type, BlurType::Gaussian, "Gaussian");
                    ui.selectable_value(blur_type, BlurType::Box, "Box");
                    ui.selectable_value(blur_type, BlurType::Directional, "Directional");
                    ui.selectable_value(blur_type, BlurType::BokehBloom, "Bokeh Bloom");
                    ui.selectable_value(blur_type, BlurType::UnsharpMask, "Unsharp Mask");
                });
            ui.horizontal(|ui| {
                ui.label("Radius:");
                drag_val(ui, DragValue::new(radius).range(0.0..=100.0).speed(0.5));
                if *blur_type == BlurType::Directional {
                    ui.label("Angle:");
                    drag_val(ui, DragValue::new(angle_deg).range(0.0..=360.0).suffix("°"));
                }
            });
            if *blur_type == BlurType::BokehBloom {
                ui.horizontal(|ui| {
                    ui.label("Highlight Boost:");
                    drag_val(ui, DragValue::new(highlight_boost).range(0.0..=3.0).speed(0.05));
                });
            }
        }
        DarkNode::DistortLens { distort_type, intensity, radius, center } => {
            ComboBox::from_label("Algorithm")
                .selected_text(format!("{distort_type:?}"))
                .show_ui(ui, |ui| {
                    ui.selectable_value(distort_type, DistortType::Vignette, "Vignette");
                    ui.selectable_value(distort_type, DistortType::FishEye, "FishEye");
                    ui.selectable_value(distort_type, DistortType::LensDistortion, "Lens Distortion");
                    ui.selectable_value(distort_type, DistortType::RippleWave, "Ripple / Wave");
                    ui.selectable_value(distort_type, DistortType::ChromaticAberration, "Chromatic Aberration");
                });
            ui.horizontal(|ui| {
                ui.label("Intensity:");
                drag_val(ui, DragValue::new(intensity).range(-2.0..=2.0).speed(0.02));
                ui.label("Radius:");
                drag_val(ui, DragValue::new(radius).range(0.0..=2.0).speed(0.02));
            });
            ui.horizontal(|ui| {
                ui.label("Center X:");
                drag_val(ui, DragValue::new(&mut center[0]).range(-1.0..=1.0).speed(0.01));
                ui.label("Y:");
                drag_val(ui, DragValue::new(&mut center[1]).range(-1.0..=1.0).speed(0.01));
            });
        }
        DarkNode::Transition { transition_type, progress, feather } => {
            ComboBox::from_label("Effect")
                .selected_text(format!("{transition_type:?}"))
                .show_ui(ui, |ui| {
                    ui.selectable_value(transition_type, TransitionType::CrossDissolve, "Cross Dissolve");
                    ui.selectable_value(transition_type, TransitionType::LinearWipe, "Linear Wipe");
                    ui.selectable_value(transition_type, TransitionType::Slide, "Slide");
                    ui.selectable_value(transition_type, TransitionType::Iris, "Iris");
                    ui.selectable_value(transition_type, TransitionType::ZoomPush, "Zoom Push");
                });
            ui.horizontal(|ui| {
                ui.label("Progress:");
                drag_val(ui, DragValue::new(progress).range(0.0..=1.0).speed(0.01));
                ui.label("Feather:");
                drag_val(ui, DragValue::new(feather).range(0.0..=1.0).speed(0.01));
            });
        }
        DarkNode::Transform2D { position, scale, rotation_deg, anchor, opacity, flip_h, flip_v } => {
            ui.horizontal(|ui| {
                ui.label("Pos X:");
                drag_val(ui, DragValue::new(&mut position[0]));
                ui.label("Y:");
                drag_val(ui, DragValue::new(&mut position[1]));
            });
            ui.horizontal(|ui| {
                ui.label("Scale:");
                drag_val(ui, DragValue::new(&mut scale[0]).speed(0.01));
                ui.label("Rot:");
                drag_val(ui, DragValue::new(rotation_deg).suffix("°"));
            });
            ui.horizontal(|ui| {
                ui.label("Anchor X:");
                drag_val(ui, DragValue::new(&mut anchor[0]).speed(0.01));
                ui.label("Y:");
                drag_val(ui, DragValue::new(&mut anchor[1]).speed(0.01));
            });
            ui.horizontal(|ui| {
                ui.label("Opacity:");
                drag_val(ui, DragValue::new(opacity).range(0.0..=1.0).speed(0.01));
                check_box(ui, flip_h, "Flip H");
                check_box(ui, flip_v, "Flip V");
            });
        }
        DarkNode::CropMask { top, bottom, left, right, feather } => {
            ui.horizontal(|ui| {
                ui.label("Top:");
                drag_val(ui, DragValue::new(top).range(0.0..=1.0).speed(0.01));
                ui.label("Btm:");
                drag_val(ui, DragValue::new(bottom).range(0.0..=1.0).speed(0.01));
            });
            ui.horizontal(|ui| {
                ui.label("Left:");
                drag_val(ui, DragValue::new(left).range(0.0..=1.0).speed(0.01));
                ui.label("Right:");
                drag_val(ui, DragValue::new(right).range(0.0..=1.0).speed(0.01));
            });
            ui.horizontal(|ui| {
                ui.label("Feather:");
                drag_val(ui, DragValue::new(feather).range(0.0..=1.0).speed(0.01));
            });
        }
        DarkNode::Keyer { key_color, similarity, smoothness, spill_suppress } => {
            ui.horizontal(|ui| {
                ui.label("Similarity:");
                drag_val(ui, DragValue::new(similarity).range(0.0..=1.0).speed(0.01));
                ui.label("Smooth:");
                drag_val(ui, DragValue::new(smoothness).range(0.0..=1.0).speed(0.01));
            });
            ui.horizontal(|ui| {
                ui.label("Spill Suppress:");
                drag_val(ui, DragValue::new(spill_suppress).range(0.0..=1.0).speed(0.01));
            });
            ui.horizontal(|ui| {
                ui.label("Key Color RGB:");
                drag_val(ui, DragValue::new(&mut key_color[0]).range(0.0..=1.0).speed(0.01));
                drag_val(ui, DragValue::new(&mut key_color[1]).range(0.0..=1.0).speed(0.01));
                drag_val(ui, DragValue::new(&mut key_color[2]).range(0.0..=1.0).speed(0.01));
            });
        }
        DarkNode::Blend { opacity, .. } => {
            ui.horizontal(|ui| {
                ui.label("Opacity:");
                drag_val(ui, DragValue::new(opacity).range(0.0..=1.0).speed(0.01));
            });
        }
        DarkNode::ActionNode { is_running, scope, .. } => {
            ui.horizontal(|ui| {
                ui.label(format!("Scope: {scope:?}"));
            });
            let btn_text = if *is_running { "⏳ Running..." } else { "▶ RUN WORKFLOW" };
            if btn(ui, btn_text).clicked() {
                *is_running = !*is_running;
            }
        }
        DarkNode::StickyNote { content, .. } => {
            text_edit_multi(ui, content);
        }
        DarkNode::CustomWgsl { code, .. } => {
            ui.label("WGSL Compute Shader:");
            text_edit_multi(ui, code);
        }
        _ => {}
    }
}
