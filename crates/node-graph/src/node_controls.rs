//! Bộ điều khiển giao diện UI Controls cho các Node trong Darkworks Node Graph.
//!
//! Thiết kế bố cục gọn theo chiều dọc (~200px) tránh che phủ màn hình,
//! tích hợp Color Picker trực tiếp trên `TextGenerator`, `ShapeGenerator`, `ColorGrade` và `Keyer`.

use std::sync::OnceLock;

use egui::{ComboBox, CursorIcon, DragValue, Grid, Response, TextEdit, Ui};
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

pub(crate) fn color_picker_rgba(ui: &mut Ui, color: &mut [f32; 4]) {
    let r = ui.color_edit_button_rgba_unmultiplied(color);
    if r.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
}

/// Render toàn bộ thông số của các Atomic Nodes theo chiều dọc gọn gàng.
pub fn render_node_body(node: &mut DarkNode, ui: &mut Ui) {
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
                    color_picker_rgba(ui, color);
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
                        color_picker_rgba(ui, stroke_color);
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
                    color_picker_rgba(ui, color);
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
                        color_picker_rgba(ui, stroke_color);
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
            } => {
                Grid::new(("xf_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("Pos X / Y:");
                    ui.horizontal(|ui| {
                        drag_val(ui, DragValue::new(&mut position[0]));
                        drag_val(ui, DragValue::new(&mut position[1]));
                    });
                    ui.end_row();
                    ui.label("Scale X/Y:");
                    ui.horizontal(|ui| {
                        drag_val(ui, DragValue::new(&mut scale[0]).speed(0.01));
                        drag_val(ui, DragValue::new(&mut scale[1]).speed(0.01));
                    });
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
                crate::node_controls_fx::render_fx_controls(node, node_ptr, ui);
            }
        }
    });
}
