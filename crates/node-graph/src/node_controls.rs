//! Bộ điều khiển giao diện UI Controls cho các Node trong Darkworks Node Graph.
//!
//! Thiết kế bố cục gọn theo chiều dọc (~200px) tránh che phủ màn hình,
//! tích hợp Color Picker trực tiếp trên `TextGenerator`, `ShapeGenerator`, `ColorGrade` và `Keyer`.

use std::sync::OnceLock;

use egui::{Color32, ComboBox, CursorIcon, DragValue, Grid, Pos2, Response, Stroke, TextEdit, Ui, vec2};
use egui_snarl::NodeId;
use crate::node::{
    ActionScope, AudioBand, BlendMode, DarkNode, ShapeType, TextAlign, TimeMode,
};

fn font_family_names() -> &'static [String] {
    static FAMILIES: OnceLock<Vec<String>> = OnceLock::new();
    FAMILIES.get_or_init(|| {
        filmcraft_text::fonts::families()
            .into_iter()
            .map(|(name, _)| name)
            .collect()
    })
}

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

/// Yêu cầu kích hoạt Eyedropper phát sinh từ nút bấm trên Node UI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeEyedropperRequest {
    pub node_id: NodeId,
    pub param: &'static str,
}

/// Trạng thái Eyedropper đang bật để tô sáng nút ống hút trên Node UI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeEyedropperActive {
    pub node_id: NodeId,
    pub param: String,
}

#[must_use]
pub fn eyedropper_request_id() -> egui::Id {
    egui::Id::unique("darkworks-node-eyedropper-req")
}

#[must_use]
pub fn eyedropper_active_id() -> egui::Id {
    egui::Id::unique("darkworks-node-eyedropper-active")
}

fn draw_eyedropper_btn(ui: &mut Ui, node_id: NodeId, param: &'static str) {
    let active = ui
        .ctx()
        .data(|d| d.get_temp::<NodeEyedropperActive>(eyedropper_active_id()))
        .is_some_and(|a| a.node_id == node_id && a.param == param);
    let (rect, resp) = ui.allocate_exact_size(vec2(18.0, 18.0), egui::Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let bg = if active {
            Color32::from_rgb(0, 165, 220)
        } else if resp.hovered() {
            Color32::from_gray(72)
        } else {
            Color32::from_gray(46)
        };
        p.rect_filled(rect, 3.0, bg);
        let fg = if active { Color32::BLACK } else { Color32::from_gray(230) };
        let c = rect.center();
        // Vẽ ống hút màu vector nghiêng 45°
        p.line_segment([c + vec2(-4.5, 4.5), c + vec2(1.5, -1.5)], Stroke::new(1.6, fg));
        p.line_segment([c + vec2(0.0, -3.0), c + vec2(3.0, 0.0)], Stroke::new(1.4, fg));
        p.line_segment([c + vec2(1.5, -1.5), c + vec2(4.2, -4.2)], Stroke::new(2.2, fg));
        p.circle_filled(Pos2::new(c.x - 4.8, c.y + 4.8), 1.0, fg);
    }
    let resp = resp.on_hover_text("Hút màu từ Program Monitor (Esc hủy)");
    if resp.clicked() {
        ui.ctx().data_mut(|d| {
            d.insert_temp(eyedropper_request_id(), NodeEyedropperRequest { node_id, param });
        });
    }
}

pub(crate) fn color_picker_rgba_ex(ui: &mut Ui, node_id: NodeId, param: &'static str, color: &mut [f32; 4]) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let r = ui.color_edit_button_rgba_unmultiplied(color);
        if r.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        draw_eyedropper_btn(ui, node_id, param);
    });
}

pub(crate) fn color_picker_rgb_ex(ui: &mut Ui, node_id: NodeId, param: &'static str, color: &mut [f32; 3]) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let r = ui.color_edit_button_rgb(color);
        if r.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        draw_eyedropper_btn(ui, node_id, param);
    });
}

/// Render toàn bộ thông số của các Atomic Nodes theo chiều dọc gọn gàng.
pub fn render_node_body(node: &mut DarkNode, ui: &mut Ui) {
    render_node_body_for(NodeId(0), node, ui);
}

/// Render toàn bộ thông số của Node kèm `NodeId` để gắn Eyedropper theo từng node/tham số.
pub fn render_node_body_for(node_id: NodeId, node: &mut DarkNode, ui: &mut Ui) {
    let node_ptr = std::ptr::from_ref(node) as usize;
    ui.vertical(|ui| {
        ui.set_max_width(210.0);
        match node {
            DarkNode::VideoInput { track_name } => {
                Grid::new(("vin_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Track:");
                    let display = match track_name.as_str() {
                        "Auto" | "Clip" => "Auto (Clip)",
                        "All Below" => "All Below",
                        other => other,
                    };
                    ComboBox::from_id_salt(("vin_track", node_ptr))
                        .selected_text(display)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(track_name, "Auto".to_string(), "Auto (Clip)");
                            ui.selectable_value(track_name, "All Below".to_string(), "All Below");
                            for t in ["V1", "V2", "V3", "V4"] {
                                ui.selectable_value(track_name, t.to_string(), t);
                            }
                        });
                    ui.end_row();
                });
            }
            DarkNode::TextGenerator {
                text,
                font_size,
                color,
                tracking,
                line_spacing,
                position,
                align,
                font_family,
                bold,
                italic,
                stroke_color,
                stroke_width,
            } => {
                ui.label("Text:");
                text_edit_multi(ui, text);
                Grid::new(("txt_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Color:");
                    color_picker_rgba_ex(ui, node_id, "text.color", color);
                    ui.end_row();
                    ui.label("Size:");
                    drag_val(ui, DragValue::new(font_size).range(8.0..=200.0));
                    ui.end_row();
                    ui.label("Font:");
                    let font_label = if font_family.trim().is_empty() {
                        "Default"
                    } else {
                        font_family.as_str()
                    };
                    ComboBox::from_id_salt(("txt_font", node_ptr))
                        .width(120.0)
                        .selected_text(font_label)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(font_family, String::new(), "Default");
                            for fam in font_family_names() {
                                ui.selectable_value(font_family, fam.clone(), fam);
                            }
                        });
                    ui.end_row();
                    ui.label("Align:");
                    ComboBox::from_id_salt(("txt_align", node_ptr))
                        .selected_text(format!("{align:?}"))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(align, TextAlign::Left, "Left");
                            ui.selectable_value(align, TextAlign::Center, "Center");
                            ui.selectable_value(align, TextAlign::Right, "Right");
                        });
                    ui.end_row();
                    ui.label("Pos X/Y:");
                    ui.horizontal(|ui| {
                        drag_val(ui, DragValue::new(&mut position[0]).speed(1.0));
                        drag_val(ui, DragValue::new(&mut position[1]).speed(1.0));
                    });
                    ui.end_row();
                    ui.label("Track:");
                    drag_val(ui, DragValue::new(tracking).range(-10.0..=50.0).speed(0.1));
                    ui.end_row();
                    ui.label("Line:");
                    drag_val(ui, DragValue::new(line_spacing).range(0.5..=3.0).speed(0.05));
                    ui.end_row();
                    ui.label("Stroke:");
                    ui.horizontal(|ui| {
                        drag_val(ui, DragValue::new(stroke_width).range(0.0..=20.0).speed(0.2));
                        color_picker_rgba_ex(ui, node_id, "text.stroke_color", stroke_color);
                    });
                    ui.end_row();
                });
                ui.horizontal(|ui| {
                    check_box(ui, bold, "Bold");
                    check_box(ui, italic, "Italic");
                });
            }
            DarkNode::ShapeGenerator {
                shape_type,
                size,
                corner_radius,
                color,
                position,
                stroke_width,
                stroke_color,
                feather,
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
                    color_picker_rgba_ex(ui, node_id, "shape.color", color);
                    ui.end_row();
                    ui.label("Pos X/Y:");
                    ui.horizontal(|ui| {
                        drag_val(ui, DragValue::new(&mut position[0]).speed(1.0));
                        drag_val(ui, DragValue::new(&mut position[1]).speed(1.0));
                    });
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
                    ui.label("Feather:");
                    drag_val(ui, DragValue::new(feather).range(0.0..=200.0).speed(0.5));
                    ui.end_row();
                    ui.label("Stroke:");
                    ui.horizontal(|ui| {
                        drag_val(ui, DragValue::new(stroke_width).range(0.0..=50.0).speed(0.2));
                        color_picker_rgba_ex(ui, node_id, "shape.stroke_color", stroke_color);
                    });
                    ui.end_row();
                });
            }
            DarkNode::TimeDriver {
                speed,
                mode,
                amplitude,
                offset,
                vec2_axis,
            } => {
                Grid::new(("tm_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    let mode_label = match mode {
                        TimeMode::Linear => "Linear",
                        TimeMode::SineLoop => "SineLoop",
                        TimeMode::PingPong => "PingPong",
                        TimeMode::Clip01 => "Clip 0→1",
                    };
                    ui.label("Mode:");
                    ComboBox::from_id_salt(("tm_mode", node_ptr))
                        .selected_text(mode_label)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(mode, TimeMode::Linear, "Linear (giây)");
                            ui.selectable_value(mode, TimeMode::Clip01, "Clip 0→1 (theo Clip)");
                            ui.selectable_value(mode, TimeMode::SineLoop, "SineLoop");
                            ui.selectable_value(mode, TimeMode::PingPong, "PingPong");
                        });
                    ui.end_row();
                    if *mode == TimeMode::Clip01 {
                        ui.label("Range:");
                        ui.weak("0.0 → 1.0 (theo Clip)");
                        ui.end_row();
                    } else {
                        let mut dur_s = if *speed > 1e-4 { 1.0 / *speed } else { 1.0 };
                        ui.label("Duration (s):");
                        if drag_val(
                            ui,
                            DragValue::new(&mut dur_s).range(0.05..=600.0).speed(0.05).suffix("s"),
                        )
                        .changed()
                        {
                            *speed = (1.0 / dur_s.max(0.05)).clamp(0.001, 200.0);
                        }
                        ui.end_row();
                        ui.label("Speed:");
                        drag_val(ui, DragValue::new(speed).range(0.001..=200.0).speed(0.05));
                        ui.end_row();
                    }
                    ui.label("Amp:");
                    drag_val(ui, DragValue::new(amplitude).speed(0.05));
                    ui.end_row();
                    ui.label("Offset:");
                    drag_val(ui, DragValue::new(offset).speed(0.05));
                    ui.end_row();
                    ui.label("Vec2 X/Y:");
                    ui.horizontal(|ui| {
                        drag_val(ui, DragValue::new(&mut vec2_axis[0]).speed(1.0));
                        drag_val(ui, DragValue::new(&mut vec2_axis[1]).speed(1.0));
                    });
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
                uniform_scale,
                skew,
                skew_axis,
            } => {
                Grid::new(("xf_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Pos X / Y:");
                    ui.horizontal(|ui| {
                        drag_val(ui, DragValue::new(&mut position[0]));
                        drag_val(ui, DragValue::new(&mut position[1]));
                    });
                    ui.end_row();
                    if *uniform_scale {
                        ui.label("Scale:");
                        if drag_val(ui, DragValue::new(&mut scale[0]).speed(0.01)).changed() {
                            scale[1] = scale[0];
                        }
                    } else {
                        ui.label("Scale X/Y:");
                        ui.horizontal(|ui| {
                            drag_val(ui, DragValue::new(&mut scale[0]).speed(0.01));
                            drag_val(ui, DragValue::new(&mut scale[1]).speed(0.01));
                        });
                    }
                    ui.end_row();
                    ui.label("Uniform:");
                    if check_box(ui, uniform_scale, "Lock X = Y").changed() && *uniform_scale {
                        scale[1] = scale[0];
                    }
                    ui.end_row();
                    ui.label("Rotation:");
                    drag_val(ui, DragValue::new(rotation_deg).suffix("°"));
                    ui.end_row();
                    ui.label("Skew/Axis:");
                    ui.horizontal(|ui| {
                        drag_val(ui, DragValue::new(skew).range(-70.0..=70.0).speed(0.2).suffix("°"));
                        drag_val(ui, DragValue::new(skew_axis).range(-360.0..=360.0).speed(0.5).suffix("°"));
                    });
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
                vec2_axis,
                track_name,
            } => {
                Grid::new(("au_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Track:");
                    let track_label = match track_name.as_str() {
                        "Master" | "" => "Master Mix",
                        "Linked" => "Linked Audio",
                        other => other,
                    };
                    ComboBox::from_id_salt(("au_track", node_ptr))
                        .selected_text(track_label)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(track_name, "Master".to_string(), "Master Mix");
                            ui.selectable_value(track_name, "Linked".to_string(), "Linked Audio");
                            for t in ["A1", "A2", "A3", "A4"] {
                                ui.selectable_value(track_name, t.to_string(), t);
                            }
                        });
                    ui.end_row();
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
                    ui.label("Vec2 X/Y:");
                    ui.horizontal(|ui| {
                        drag_val(ui, DragValue::new(&mut vec2_axis[0]).speed(1.0));
                        drag_val(ui, DragValue::new(&mut vec2_axis[1]).speed(1.0));
                    });
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
            DarkNode::Reroute { data_type } => {
                use crate::types::NodeDataType;
                ComboBox::from_id_salt(("reroute_ty", node_ptr))
                    .width(92.0)
                    .selected_text(data_type.name())
                    .show_ui(ui, |ui| {
                        for ty in [
                            NodeDataType::Any,
                            NodeDataType::Texture,
                            NodeDataType::Scalar,
                            NodeDataType::Vec2,
                            NodeDataType::AudioFloat,
                        ] {
                            ui.selectable_value(data_type, ty, ty.name());
                        }
                    });
            }
            _ => {
                crate::node_controls_fx::render_fx_controls(node_id, node, node_ptr, ui);
            }
        }
    });
}
