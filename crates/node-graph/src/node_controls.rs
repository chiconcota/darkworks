//! Bộ điều khiển giao diện UI Controls cho các Node trong Darkworks Node Graph.
//!
//! Thiết kế bố cục gọn theo chiều dọc (~200px) tránh che phủ màn hình,
//! tích hợp Color Picker trực tiếp trên `TextGenerator`, `ShapeGenerator`, `ColorGrade` và `Keyer`.

use egui::{ComboBox, CursorIcon, DragValue, Grid, Response, TextEdit, Ui};
use crate::node::{
    ActionScope, AudioBand, BlendMode, BlurType, DarkNode, DistortType, ShapeType, TimeMode,
    TransitionType,
};

#[inline]
pub fn drag_val(ui: &mut Ui, dv: DragValue<'_>) -> Response {
    let r = ui.add(dv);
    if r.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
    }
    r
}

#[inline]
pub fn text_edit(ui: &mut Ui, text: &mut String) -> Response {
    let r = ui.add(TextEdit::singleline(text).desired_width(175.0));
    if r.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::Text);
    }
    r
}

#[inline]
pub fn text_edit_multi(ui: &mut Ui, text: &mut String) -> Response {
    let r = ui.add(TextEdit::multiline(text).desired_width(180.0).desired_rows(2));
    if r.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::Text);
    }
    r
}

#[inline]
pub fn check_box(ui: &mut Ui, checked: &mut bool, text: &str) -> Response {
    let r = ui.checkbox(checked, text);
    if r.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    r
}

#[inline]
pub fn btn(ui: &mut Ui, text: &str) -> Response {
    let r = ui.button(text);
    if r.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    r
}

fn color_picker_rgba(ui: &mut Ui, color: &mut [f32; 4]) {
    let r = ui.color_edit_button_rgba_unmultiplied(color);
    if r.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
}

/// Render toàn bộ thông số của các Atomic Nodes theo chiều dọc gọn gàng.
pub fn render_node_body(node: &mut DarkNode, ui: &mut Ui) {
    let node_ptr = std::ptr::from_ref(node) as usize;
    ui.scope(|ui| {
        ui.set_max_width(195.0);
        match node {
            DarkNode::TextGenerator {
                text,
                font_size,
                color,
                tracking,
                line_spacing,
            } => {
                ui.label("Text:");
                text_edit_multi(ui, text);
                Grid::new(("txt_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Color:");
                    color_picker_rgba(ui, color);
                    ui.end_row();
                    ui.label("Size:");
                    drag_val(ui, DragValue::new(font_size).range(8.0..=200.0));
                    ui.end_row();
                    ui.label("Track:");
                    drag_val(ui, DragValue::new(tracking).range(-10.0..=50.0).speed(0.1));
                    ui.end_row();
                    ui.label("Line:");
                    drag_val(ui, DragValue::new(line_spacing).range(0.5..=3.0).speed(0.05));
                    ui.end_row();
                });
            }
            DarkNode::ShapeGenerator {
                shape_type,
                size,
                corner_radius,
                color,
            } => {
                Grid::new(("shp_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Shape:");
                    ComboBox::from_id_salt(("shp_type", node_ptr))
                        .selected_text(format!("{shape_type:?}"))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(shape_type, ShapeType::Rectangle, "Rectangle");
                            ui.selectable_value(shape_type, ShapeType::RoundedRectangle, "Rounded Rect");
                            ui.selectable_value(shape_type, ShapeType::Circle, "Circle");
                            ui.selectable_value(shape_type, ShapeType::SolidMatte, "Solid Matte");
                        });
                    ui.end_row();
                    ui.label("Color:");
                    color_picker_rgba(ui, color);
                    ui.end_row();
                    ui.label("Width:");
                    drag_val(ui, DragValue::new(&mut size[0]).range(1.0..=3840.0));
                    ui.end_row();
                    ui.label("Height:");
                    drag_val(ui, DragValue::new(&mut size[1]).range(1.0..=2160.0));
                    ui.end_row();
                    if *shape_type == ShapeType::RoundedRectangle {
                        ui.label("Radius:");
                        drag_val(ui, DragValue::new(corner_radius).range(0.0..=100.0));
                        ui.end_row();
                    }
                });
            }
            DarkNode::TimeDriver { speed, mode } => {
                Grid::new(("tm_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Speed:");
                    drag_val(ui, DragValue::new(speed).speed(0.1));
                    ui.end_row();
                    ui.label("Mode:");
                    ComboBox::from_id_salt(("tm_mode", node_ptr))
                        .selected_text(format!("{mode:?}"))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(mode, TimeMode::Linear, "Linear");
                            ui.selectable_value(mode, TimeMode::SineLoop, "SineLoop");
                            ui.selectable_value(mode, TimeMode::PingPong, "PingPong");
                        });
                    ui.end_row();
                });
            }
            DarkNode::ColorGrade {
                exposure,
                contrast,
                saturation,
                temperature,
                tint,
                gamma,
                invert,
                black_and_white,
                tint_color,
                tint_amount,
            } => {
                Grid::new(("cg_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Exp:");
                    drag_val(ui, DragValue::new(exposure).range(-5.0..=5.0).speed(0.05));
                    ui.end_row();
                    ui.label("Contrast:");
                    drag_val(ui, DragValue::new(contrast).range(0.0..=3.0).speed(0.05));
                    ui.end_row();
                    ui.label("Sat:");
                    drag_val(ui, DragValue::new(saturation).range(0.0..=3.0).speed(0.05));
                    ui.end_row();
                    ui.label("Temp:");
                    drag_val(ui, DragValue::new(temperature).range(-100.0..=100.0).speed(0.5));
                    ui.end_row();
                    ui.label("Tint:");
                    drag_val(ui, DragValue::new(tint).range(-100.0..=100.0).speed(0.5));
                    ui.end_row();
                    ui.label("Gamma:");
                    drag_val(ui, DragValue::new(gamma).range(0.1..=4.0).speed(0.05));
                    ui.end_row();
                    ui.label("Tint Color:");
                    color_picker_rgba(ui, tint_color);
                    ui.end_row();
                    ui.label("Tint Mix:");
                    drag_val(ui, DragValue::new(tint_amount).range(0.0..=1.0).speed(0.02));
                    ui.end_row();
                });
                check_box(ui, invert, "⚡ Invert Colors");
                check_box(ui, black_and_white, "⬛ B&W");
            }
            DarkNode::BlurSharpen {
                blur_type,
                radius,
                angle_deg,
                highlight_boost,
            } => {
                ComboBox::from_id_salt(("blur_type", node_ptr))
                    .selected_text(format!("{blur_type:?}"))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(blur_type, BlurType::Gaussian, "Gaussian");
                        ui.selectable_value(blur_type, BlurType::Box, "Box");
                        ui.selectable_value(blur_type, BlurType::Directional, "Directional");
                        ui.selectable_value(blur_type, BlurType::BokehBloom, "Bokeh Bloom");
                        ui.selectable_value(blur_type, BlurType::UnsharpMask, "Unsharp Mask");
                    });
                Grid::new(("blur_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Radius:");
                    drag_val(ui, DragValue::new(radius).range(0.0..=100.0).speed(0.5));
                    ui.end_row();
                    if *blur_type == BlurType::Directional {
                        ui.label("Angle:");
                        drag_val(ui, DragValue::new(angle_deg).range(0.0..=360.0).suffix("°"));
                        ui.end_row();
                    }
                    if *blur_type == BlurType::BokehBloom {
                        ui.label("Boost:");
                        drag_val(ui, DragValue::new(highlight_boost).range(0.0..=3.0).speed(0.05));
                        ui.end_row();
                    }
                });
            }
            DarkNode::DistortLens {
                distort_type,
                intensity,
                radius,
                center,
            } => {
                ComboBox::from_id_salt(("dist_type", node_ptr))
                    .selected_text(format!("{distort_type:?}"))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(distort_type, DistortType::Vignette, "Vignette");
                        ui.selectable_value(distort_type, DistortType::FishEye, "FishEye");
                        ui.selectable_value(distort_type, DistortType::LensDistortion, "Lens Distortion");
                        ui.selectable_value(distort_type, DistortType::RippleWave, "Ripple / Wave");
                        ui.selectable_value(distort_type, DistortType::ChromaticAberration, "Chromatic Aberration");
                    });
                Grid::new(("dist_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Intensity:");
                    drag_val(ui, DragValue::new(intensity).range(-2.0..=2.0).speed(0.02));
                    ui.end_row();
                    ui.label("Radius:");
                    drag_val(ui, DragValue::new(radius).range(0.0..=2.0).speed(0.02));
                    ui.end_row();
                    ui.label("Center X:");
                    drag_val(ui, DragValue::new(&mut center[0]).range(-1.0..=1.0).speed(0.01));
                    ui.end_row();
                    ui.label("Center Y:");
                    drag_val(ui, DragValue::new(&mut center[1]).range(-1.0..=1.0).speed(0.01));
                    ui.end_row();
                });
            }
            DarkNode::Transition {
                transition_type,
                progress,
                feather,
            } => {
                ComboBox::from_id_salt(("tr_type", node_ptr))
                    .selected_text(format!("{transition_type:?}"))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(transition_type, TransitionType::CrossDissolve, "Cross Dissolve");
                        ui.selectable_value(transition_type, TransitionType::LinearWipe, "Linear Wipe");
                        ui.selectable_value(transition_type, TransitionType::Slide, "Slide");
                        ui.selectable_value(transition_type, TransitionType::Iris, "Iris");
                        ui.selectable_value(transition_type, TransitionType::ZoomPush, "Zoom Push");
                    });
                Grid::new(("tr_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Progress:");
                    drag_val(ui, DragValue::new(progress).range(0.0..=1.0).speed(0.01));
                    ui.end_row();
                    ui.label("Feather:");
                    drag_val(ui, DragValue::new(feather).range(0.0..=1.0).speed(0.01));
                    ui.end_row();
                });
            }
            DarkNode::Transform2D {
                position,
                scale,
                rotation_deg,
                anchor,
                opacity,
                flip_h,
                flip_v,
            } => {
                Grid::new(("xf_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Pos X / Y:");
                    ui.horizontal(|ui| {
                        drag_val(ui, DragValue::new(&mut position[0]));
                        drag_val(ui, DragValue::new(&mut position[1]));
                    });
                    ui.end_row();
                    ui.label("Scale:");
                    drag_val(ui, DragValue::new(&mut scale[0]).speed(0.01));
                    ui.end_row();
                    ui.label("Rotation:");
                    drag_val(ui, DragValue::new(rotation_deg).suffix("°"));
                    ui.end_row();
                    ui.label("Anchor:");
                    ui.horizontal(|ui| {
                        drag_val(ui, DragValue::new(&mut anchor[0]).speed(0.01));
                        drag_val(ui, DragValue::new(&mut anchor[1]).speed(0.01));
                    });
                    ui.end_row();
                    ui.label("Opacity:");
                    drag_val(ui, DragValue::new(opacity).range(0.0..=1.0).speed(0.01));
                    ui.end_row();
                });
                ui.horizontal(|ui| {
                    check_box(ui, flip_h, "Flip H");
                    check_box(ui, flip_v, "Flip V");
                });
            }
            DarkNode::CropMask {
                top,
                bottom,
                left,
                right,
                feather,
            } => {
                Grid::new(("cr_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Top:");
                    drag_val(ui, DragValue::new(top).range(0.0..=1.0).speed(0.01));
                    ui.end_row();
                    ui.label("Bottom:");
                    drag_val(ui, DragValue::new(bottom).range(0.0..=1.0).speed(0.01));
                    ui.end_row();
                    ui.label("Left:");
                    drag_val(ui, DragValue::new(left).range(0.0..=1.0).speed(0.01));
                    ui.end_row();
                    ui.label("Right:");
                    drag_val(ui, DragValue::new(right).range(0.0..=1.0).speed(0.01));
                    ui.end_row();
                    ui.label("Feather:");
                    drag_val(ui, DragValue::new(feather).range(0.0..=1.0).speed(0.01));
                    ui.end_row();
                });
            }
            DarkNode::Keyer {
                key_color,
                similarity,
                smoothness,
                spill_suppress,
            } => {
                Grid::new(("ky_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Key Color:");
                    ui.color_edit_button_rgb(key_color);
                    ui.end_row();
                    ui.label("Similarity:");
                    drag_val(ui, DragValue::new(similarity).range(0.0..=1.0).speed(0.01));
                    ui.end_row();
                    ui.label("Smooth:");
                    drag_val(ui, DragValue::new(smoothness).range(0.0..=1.0).speed(0.01));
                    ui.end_row();
                    ui.label("Spill:");
                    drag_val(ui, DragValue::new(spill_suppress).range(0.0..=1.0).speed(0.01));
                    ui.end_row();
                });
            }
            DarkNode::Blend { mode, opacity } => {
                Grid::new(("bl_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Mode:");
                    ComboBox::from_id_salt(("bl_mode", node_ptr))
                        .selected_text(mode.label())
                        .show_ui(ui, |ui| {
                            for &m in BlendMode::ALL {
                                ui.selectable_value(mode, m, m.label());
                            }
                        });
                    ui.end_row();
                    ui.label("Opacity:");
                    drag_val(ui, DragValue::new(opacity).range(0.0..=1.0).speed(0.01));
                    ui.end_row();
                });
            }
            DarkNode::AudioReactor {
                band,
                multiplier,
                smooth,
            } => {
                Grid::new(("au_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Band:");
                    ComboBox::from_id_salt(("au_band", node_ptr))
                        .selected_text(format!("{band:?}"))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(band, AudioBand::SubBass, "SubBass");
                            ui.selectable_value(band, AudioBand::Bass, "Bass");
                            ui.selectable_value(band, AudioBand::LowMid, "LowMid");
                            ui.selectable_value(band, AudioBand::HighMid, "HighMid");
                            ui.selectable_value(band, AudioBand::Treble, "Treble");
                            ui.selectable_value(band, AudioBand::MasterPeak, "MasterPeak");
                        });
                    ui.end_row();
                    ui.label("Mult:");
                    drag_val(ui, DragValue::new(multiplier).range(0.1..=10.0).speed(0.05));
                    ui.end_row();
                    ui.label("Smooth:");
                    drag_val(ui, DragValue::new(smooth).range(0.0..=1.0).speed(0.01));
                    ui.end_row();
                });
            }
            DarkNode::ActionNode { is_running, scope, .. } => {
                Grid::new(("act_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Scope:");
                    ComboBox::from_id_salt(("act_scope", node_ptr))
                        .selected_text(format!("{scope:?}"))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(scope, ActionScope::MarkedInOut, "Marked In/Out");
                            ui.selectable_value(scope, ActionScope::SelectedClips, "Selected Clips");
                            ui.selectable_value(scope, ActionScope::TrackV1, "Track V1");
                            ui.selectable_value(scope, ActionScope::TrackV2, "Track V2");
                            ui.selectable_value(scope, ActionScope::AllTracks, "All Tracks");
                        });
                    ui.end_row();
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
                ui.label("WGSL Fragment:");
                text_edit_multi(ui, code);
            }
            _ => {}
        }
    });
}
