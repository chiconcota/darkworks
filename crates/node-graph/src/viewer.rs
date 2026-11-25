//! Trình kết xuất giao diện SnarlViewer cho Darkworks Node Graph.
//!
//! Hiển thị đầy đủ UI controls, chân cắm màu sắc và tham số thời gian thực cho 18 Atomic Nodes.

use std::collections::HashMap;
use egui::{Color32, CursorIcon, Pos2, Rect, RichText, Ui, vec2};
use egui_snarl::{
    ui::{PinInfo, SnarlViewer},
    InPin, NodeId, OutPin, Snarl,
};
use crate::node::{
    BlurType, DarkNode, DistortType, ShapeType, TimeMode, TransitionType,
};
use crate::types::NodeDataType;

/// Hành động zoom từ giao diện Toolbar
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ZoomAction {
    ZoomIn,
    ZoomOut,
    Reset100,
    FitAll,
}

/// Viewer điều khiển cách vẽ và tương tác với các Node của Darkworks.
#[derive(Clone, Debug, Default)]
pub struct DarkworksViewer {
    pub zoom_action: Option<ZoomAction>,
    pub current_scale: f32,
    pub to_global: egui::emath::TSTransform,
    pub is_dragging_group: bool,
    pub prev_translation: egui::Vec2,
    pub groups: Vec<crate::group::VisualGroup>,
    pub custom_titles: HashMap<NodeId, String>,
    pub renaming_node: Option<NodeId>,
    pub renaming_buf: String,
    pub node_to_remove: Option<NodeId>,
}

impl DarkworksViewer {
    pub fn new() -> Self {
        Self {
            zoom_action: None,
            current_scale: 1.0,
            to_global: egui::emath::TSTransform::IDENTITY,
            is_dragging_group: false,
            prev_translation: egui::Vec2::ZERO,
            groups: Vec::new(),
            custom_titles: HashMap::new(),
            renaming_node: None,
            renaming_buf: String::new(),
            node_to_remove: None,
        }
    }

    pub fn with_zoom(zoom_action: Option<ZoomAction>, current_scale: f32) -> Self {
        Self {
            zoom_action,
            current_scale,
            to_global: egui::emath::TSTransform::IDENTITY,
            is_dragging_group: false,
            prev_translation: egui::Vec2::ZERO,
            groups: Vec::new(),
            custom_titles: HashMap::new(),
            renaming_node: None,
            renaming_buf: String::new(),
            node_to_remove: None,
        }
    }

    pub fn with_zoom_and_groups(
        zoom_action: Option<ZoomAction>,
        current_scale: f32,
        groups: Vec<crate::group::VisualGroup>,
    ) -> Self {
        Self {
            zoom_action,
            current_scale,
            to_global: egui::emath::TSTransform::IDENTITY,
            is_dragging_group: false,
            prev_translation: egui::Vec2::ZERO,
            groups,
            custom_titles: HashMap::new(),
            renaming_node: None,
            renaming_buf: String::new(),
            node_to_remove: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_full_state(
        zoom_action: Option<ZoomAction>,
        current_scale: f32,
        to_global: egui::emath::TSTransform,
        is_dragging_group: bool,
        groups: Vec<crate::group::VisualGroup>,
        custom_titles: HashMap<NodeId, String>,
        renaming_node: Option<NodeId>,
        renaming_buf: String,
    ) -> Self {
        Self {
            zoom_action,
            current_scale,
            to_global,
            is_dragging_group,
            prev_translation: to_global.translation,
            groups,
            custom_titles,
            renaming_node,
            renaming_buf,
            node_to_remove: None,
        }
    }
}

#[allow(refining_impl_trait)]
impl SnarlViewer<DarkNode> for DarkworksViewer {
    fn title(&mut self, node: &DarkNode) -> String {
        node.title()
    }

    fn has_node_menu(&mut self, _node: &DarkNode) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<DarkNode>,
    ) {
        let def_title = snarl[node].title();
        let display = match self.custom_titles.get(&node) {
            Some(custom) => format!("{custom} ({def_title})"),
            None => def_title,
        };
        ui.label(RichText::new(display).strong());
        ui.separator();

        if ui.button("✏️ Đổi tên (Rename)").clicked() {
            self.renaming_node = Some(node);
            self.renaming_buf = self.custom_titles.get(&node).cloned().unwrap_or_else(|| snarl[node].title());
            ui.close();
        }

        if self.custom_titles.contains_key(&node) && ui.button("🔄 Khôi phục tên gốc").clicked() {
            self.custom_titles.remove(&node);
            ui.close();
        }

        if ui.button("📑 Nhân bản (Duplicate)").clicked() {
            let clone_node = snarl[node].clone();
            let pos = snarl.get_node_info(node).map(|n| n.pos + vec2(40.0, 40.0)).unwrap_or(Pos2::ZERO);
            let new_node = snarl.insert_node(pos, clone_node);
            if let Some(custom) = self.custom_titles.get(&node).cloned() {
                self.custom_titles.insert(new_node, format!("{custom} Copy"));
            }
            ui.close();
        }

        ui.separator();

        if !inputs.is_empty() && ui.button("✂ Ngắt dây In").clicked() {
            for input in inputs {
                snarl.drop_inputs(input.id);
            }
            ui.close();
        }

        if !outputs.is_empty() && ui.button("✂ Ngắt dây Out").clicked() {
            for output in outputs {
                snarl.drop_outputs(output.id);
            }
            ui.close();
        }

        ui.separator();

        if ui.button(RichText::new("🗑 Xóa Node (Delete)").color(Color32::from_rgb(255, 100, 100))).clicked() {
            snarl.remove_node(node);
            self.custom_titles.remove(&node);
            ui.close();
        }
    }

    fn has_graph_menu(&mut self, _pos: Pos2, _snarl: &mut Snarl<DarkNode>) -> bool {
        true
    }

    fn show_graph_menu(&mut self, pos: Pos2, ui: &mut Ui, snarl: &mut Snarl<DarkNode>) {
        ui.label(RichText::new("➕ Thêm Node mới").strong());
        ui.separator();

        ui.menu_button("🎬 Media & Generators", |ui| {
            if ui.button("🎬 Video Input").clicked() { snarl.insert_node(pos, DarkNode::VideoInput { track_name: "V1".into() }); ui.close(); }
            if ui.button("🔤 Text Generator").clicked() { snarl.insert_node(pos, DarkNode::TextGenerator { text: "Text".into(), font_size: 36.0, color: [1.0; 4], tracking: 0.0, line_spacing: 1.0 }); ui.close(); }
            if ui.button("🟩 Shape Generator").clicked() { snarl.insert_node(pos, DarkNode::ShapeGenerator { shape_type: ShapeType::RoundedRectangle, size: [400.0, 120.0], corner_radius: 12.0, color: [0.0, 0.8, 1.0, 1.0] }); ui.close(); }
            if ui.button("⏱️ Time Driver").clicked() { snarl.insert_node(pos, DarkNode::TimeDriver { speed: 1.0, mode: TimeMode::Linear }); ui.close(); }
            if ui.button("🖥️ Final Output").clicked() { snarl.insert_node(pos, DarkNode::Output); ui.close(); }
        });

        ui.menu_button("🎨 Color & Keying", |ui| {
            if ui.button("🎨 Color Grade").clicked() { snarl.insert_node(pos, DarkNode::ColorGrade { exposure: 0.0, contrast: 1.0, saturation: 1.0, temperature: 0.0, tint: 0.0, gamma: 1.0, invert: false, black_and_white: false, tint_color: [1.0, 1.0, 1.0, 1.0], tint_amount: 0.0 }); ui.close(); }
            if ui.button("🟢 Ultra Keyer").clicked() { snarl.insert_node(pos, DarkNode::Keyer { key_color: [0.0, 1.0, 0.0], similarity: 0.4, smoothness: 0.1, spill_suppress: 0.5 }); ui.close(); }
        });

        ui.menu_button("📐 Transform & Motion", |ui| {
            if ui.button("📐 Transform 2D").clicked() { snarl.insert_node(pos, DarkNode::Transform2D { position: [0.0, 0.0], scale: [1.0, 1.0], rotation_deg: 0.0, anchor: [0.5, 0.5], opacity: 1.0, flip_h: false, flip_v: false }); ui.close(); }
            if ui.button("💧 Blur / Sharpen").clicked() { snarl.insert_node(pos, DarkNode::BlurSharpen { blur_type: BlurType::Gaussian, radius: 10.0, angle_deg: 0.0, highlight_boost: 0.0 }); ui.close(); }
            if ui.button("🌀 Distort / Lens").clicked() { snarl.insert_node(pos, DarkNode::DistortLens { distort_type: DistortType::Vignette, intensity: 0.5, radius: 0.8, center: [0.5, 0.5] }); ui.close(); }
            if ui.button("✂️ Crop / Mask").clicked() { snarl.insert_node(pos, DarkNode::CropMask { top: 0.0, bottom: 0.0, left: 0.0, right: 0.0, feather: 0.0 }); ui.close(); }
        });

        ui.menu_button("🔀 Composite & Utilities", |ui| {
            if ui.button("🎭 Blend").clicked() { snarl.insert_node(pos, DarkNode::Blend { mode: crate::node::BlendMode::Normal, opacity: 1.0 }); ui.close(); }
            if ui.button("⚡ Transition").clicked() { snarl.insert_node(pos, DarkNode::Transition { transition_type: TransitionType::CrossDissolve, progress: 0.5, feather: 0.1 }); ui.close(); }
            if ui.button("🔊 Audio Reactor").clicked() { snarl.insert_node(pos, DarkNode::AudioReactor { band: crate::node::AudioBand::Bass, multiplier: 1.0, smooth: 0.2 }); ui.close(); }
            if ui.button("💻 Custom WGSL").clicked() { snarl.insert_node(pos, DarkNode::CustomWgsl { name: "WGSL".into(), code: "@fragment\nfn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> { return vec4<f32>(uv.x, uv.y, 0.5, 1.0); }".into() }); ui.close(); }
            if ui.button("📝 Sticky Note").clicked() { snarl.insert_node(pos, DarkNode::StickyNote { title: "Note".into(), content: "Note".into(), color_idx: 0 }); ui.close(); }
            if ui.button("● Reroute").clicked() { snarl.insert_node(pos, DarkNode::Reroute { data_type: NodeDataType::Texture }); ui.close(); }
        });
    }

    fn draw_background(
        &mut self,
        background: Option<&egui_snarl::ui::BackgroundPattern>,
        viewport: &Rect,
        snarl_style: &egui_snarl::ui::SnarlStyle,
        style: &egui::Style,
        painter: &egui::Painter,
        snarl: &Snarl<DarkNode>,
    ) {
        let _ = snarl;
        if let Some(bg) = background {
            bg.draw(viewport, snarl_style, style, painter);
        }

        // Vẽ các Visual Groups chuẩn tọa độ đồ thị (Graph space)
        for group in &self.groups {
            // 1. Thân nền group
            painter.rect_filled(group.rect(), 8.0, group.color());
            painter.rect_stroke(
                group.rect(),
                8.0,
                egui::Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(200, 200, 255, 120)),
                egui::StrokeKind::Inside,
            );

            // 2. Thanh tiêu đề kéo thả (Header Bar)
            let header = group.header_rect();
            let header_color = Color32::from_rgba_unmultiplied(
                group.color_rgba[0],
                group.color_rgba[1],
                group.color_rgba[2],
                130,
            );
            painter.rect_filled(
                header,
                egui::CornerRadius { nw: 8, ne: 8, sw: 0, se: 0 },
                header_color,
            );

            // Chữ tiêu đề (hiện thêm icon 🔒 nếu group đang bị khóa)
            let display_title = if group.is_locked {
                format!("📁 {} 🔒", group.title)
            } else {
                format!("📁 {}", group.title)
            };
            painter.text(
                header.min + vec2(10.0, 7.0),
                egui::Align2::LEFT_TOP,
                display_title,
                egui::FontId::proportional(14.0),
                egui::Color32::WHITE,
            );

            // Nút xóa nhanh "✖" ở góc phải thanh tiêu đề
            let close_rect = group.close_button_rect();
            painter.text(
                close_rect.center(),
                egui::Align2::CENTER_CENTER,
                "✖",
                egui::FontId::proportional(13.0),
                Color32::from_rgb(255, 120, 120),
            );

            // 3. Tay cầm co giãn (Resize Grip) góc dưới phải (chỉ hiện khi chưa khóa)
            if !group.is_locked {
                let max = Pos2::new(group.max[0], group.max[1]);
                painter.line_segment(
                    [max - vec2(20.0, 5.0), max - vec2(5.0, 20.0)],
                    egui::Stroke::new(1.5, Color32::from_white_alpha(150)),
                );
                painter.line_segment(
                    [max - vec2(14.0, 5.0), max - vec2(5.0, 14.0)],
                    egui::Stroke::new(1.5, Color32::from_white_alpha(150)),
                );
                painter.line_segment(
                    [max - vec2(8.0, 5.0), max - vec2(5.0, 8.0)],
                    egui::Stroke::new(1.5, Color32::from_white_alpha(150)),
                );
            }
        }
    }

    fn show_header(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<DarkNode>,
    ) {
        ui.horizontal(|ui| {
            if self.renaming_node == Some(node) {
                let text_edit = egui::TextEdit::singleline(&mut self.renaming_buf).desired_width(130.0);
                let response = ui.add(text_edit);
                response.request_focus();

                let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                let esc = ui.input(|i| i.key_pressed(egui::Key::Escape));

                let save_clicked = ui.button(RichText::new("✓").color(Color32::from_rgb(0, 220, 100)).small())
                    .on_hover_text("Lưu tên mới (Enter)").clicked();
                let cancel_clicked = ui.button(RichText::new("✕").color(Color32::from_rgb(255, 120, 120)).small())
                    .on_hover_text("Hủy bỏ (Esc)").clicked();

                if enter || save_clicked {
                    let trimmed = self.renaming_buf.trim().to_string();
                    if !trimmed.is_empty() {
                        self.custom_titles.insert(node, trimmed);
                    } else {
                        self.custom_titles.remove(&node);
                    }
                    self.renaming_node = None;
                } else if esc || cancel_clicked {
                    self.renaming_node = None;
                }
            } else {
                let default_title = self.title(&snarl[node]);
                let display_title = match self.custom_titles.get(&node) {
                    Some(custom) => format!("{custom} ({default_title})"),
                    None => default_title,
                };
                let resp = ui.label(RichText::new(display_title).strong());
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(CursorIcon::Grab);
                }
                if resp.double_clicked() {
                    self.renaming_node = Some(node);
                    self.renaming_buf = self.custom_titles.get(&node).cloned().unwrap_or_else(|| self.title(&snarl[node]));
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let del = ui.add(
                    egui::Button::new(RichText::new("✖").color(Color32::from_rgb(255, 110, 110)).small())
                        .frame(false)
                ).on_hover_text("Xóa Node");
                if del.clicked() {
                    self.node_to_remove = Some(node);
                }
            });
        });
    }

    fn current_transform(
        &mut self,
        to_global: &mut egui::emath::TSTransform,
        snarl: &mut Snarl<DarkNode>,
    ) {
        if self.is_dragging_group {
            to_global.translation = self.prev_translation;
        } else {
            self.prev_translation = to_global.translation;
        }
        self.to_global = *to_global;
        self.current_scale = to_global.scaling;

        if let Some(action) = self.zoom_action.take() {
            match action {
                ZoomAction::ZoomIn => {
                    to_global.scaling = (to_global.scaling * 1.25).clamp(0.1, 5.0);
                }
                ZoomAction::ZoomOut => {
                    to_global.scaling = (to_global.scaling * 0.8).clamp(0.1, 5.0);
                }
                ZoomAction::Reset100 => {
                    to_global.scaling = 1.0;
                }
                ZoomAction::FitAll => {
                    let mut bb = Rect::NOTHING;
                    for (id, _) in snarl.node_ids() {
                        if let Some(info) = snarl.get_node_info(id) {
                            bb.extend_with(info.pos);
                        }
                    }
                    if bb.is_finite() {
                        to_global.scaling = 1.0;
                        to_global.translation = -bb.center().to_vec2() + vec2(400.0, 300.0);
                    }
                }
            }
            self.current_scale = to_global.scaling;
        }
    }

    fn inputs(&mut self, node: &DarkNode) -> usize {
        match node {
            DarkNode::VideoInput { .. }
            | DarkNode::TextGenerator { .. }
            | DarkNode::ShapeGenerator { .. }
            | DarkNode::TimeDriver { .. }
            | DarkNode::StickyNote { .. } => 0,

            DarkNode::CropMask { .. }
            | DarkNode::ColorGrade { .. }
            | DarkNode::BlurSharpen { .. }
            | DarkNode::Keyer { .. }
            | DarkNode::Output
            | DarkNode::AudioReactor { .. } => 1,

            DarkNode::DistortLens { .. } => 3, // 0: Texture, 1: Intensity Mod, 2: Center Mod
            DarkNode::Transition { .. } => 3,  // 0: Input A, 1: Input B, 2: Progress Mod
            DarkNode::Transform2D { .. } => 5, // 0: Texture, 1: Pos Mod, 2: Scale Mod, 3: Rot Mod, 4: Opacity Mod
            DarkNode::Blend { .. } => 3,       // 0: Background, 1: Foreground, 2: Mask
            DarkNode::BlackNode { exposed_pins, .. } => exposed_pins.len(),
            DarkNode::ActionNode { .. } => 1,
            DarkNode::CustomWgsl { .. } => 2,
            DarkNode::Reroute { .. } => 1,
        }
    }

    fn outputs(&mut self, node: &DarkNode) -> usize {
        match node {
            DarkNode::Output | DarkNode::StickyNote { .. } => 0,
            DarkNode::TimeDriver { .. }
            | DarkNode::AudioReactor { .. } => 1,

            DarkNode::VideoInput { .. }
            | DarkNode::TextGenerator { .. }
            | DarkNode::ShapeGenerator { .. }
            | DarkNode::Transform2D { .. }
            | DarkNode::CropMask { .. }
            | DarkNode::Blend { .. }
            | DarkNode::ColorGrade { .. }
            | DarkNode::BlurSharpen { .. }
            | DarkNode::DistortLens { .. }
            | DarkNode::Transition { .. }
            | DarkNode::Keyer { .. }
            | DarkNode::CustomWgsl { .. }
            | DarkNode::Reroute { .. } => 1,

            DarkNode::BlackNode { .. } => 1,
            DarkNode::ActionNode { .. } => 1,
        }
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<DarkNode>,
    ) -> PinInfo {
        let node = &snarl[pin.id.node];
        let (name, data_type) = match node {
            DarkNode::Transform2D { .. } => match pin.id.input {
                0 => ("Image In", NodeDataType::Texture),
                1 => ("Pos Mod", NodeDataType::Vec2),
                2 => ("Scale Mod", NodeDataType::Scalar),
                3 => ("Rot Mod", NodeDataType::Scalar),
                _ => ("Opacity Mod", NodeDataType::Scalar),
            },
            DarkNode::CropMask { .. }
            | DarkNode::ColorGrade { .. }
            | DarkNode::BlurSharpen { .. }
            | DarkNode::Keyer { .. }
            | DarkNode::Output => ("Image In", NodeDataType::Texture),

            DarkNode::DistortLens { .. } => match pin.id.input {
                0 => ("Image In", NodeDataType::Texture),
                1 => ("Intensity Mod", NodeDataType::Scalar),
                _ => ("Center Mod", NodeDataType::Vec2),
            },

            DarkNode::Blend { .. } => match pin.id.input {
                0 => ("Background", NodeDataType::Texture),
                1 => ("Foreground", NodeDataType::Texture),
                _ => ("Mask", NodeDataType::Texture),
            },

            DarkNode::Transition { .. } => match pin.id.input {
                0 => ("Input A", NodeDataType::Texture),
                1 => ("Input B", NodeDataType::Texture),
                _ => ("Progress Mod", NodeDataType::Scalar),
            },

            DarkNode::AudioReactor { .. } => ("Audio Track", NodeDataType::AudioFloat),
            DarkNode::ActionNode { .. } => ("Trigger In", NodeDataType::ActionFlow),
            DarkNode::CustomWgsl { .. } => ("Input", NodeDataType::Texture),
            DarkNode::Reroute { data_type } => ("In", *data_type),
            _ => ("In", NodeDataType::Any),
        };

        let resp = ui.label(name);
        if resp.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        PinInfo::circle().with_fill(crate::types::pin_color32(data_type))
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<DarkNode>,
    ) -> PinInfo {
        let node = &snarl[pin.id.node];
        let (name, data_type) = match node {
            DarkNode::VideoInput { .. }
            | DarkNode::TextGenerator { .. }
            | DarkNode::ShapeGenerator { .. }
            | DarkNode::Transform2D { .. }
            | DarkNode::CropMask { .. }
            | DarkNode::Blend { .. }
            | DarkNode::ColorGrade { .. }
            | DarkNode::BlurSharpen { .. }
            | DarkNode::DistortLens { .. }
            | DarkNode::Transition { .. }
            | DarkNode::Keyer { .. }
            | DarkNode::CustomWgsl { .. } => ("Texture Out", NodeDataType::Texture),

            DarkNode::TimeDriver { .. } => ("Time Value", NodeDataType::Scalar),
            DarkNode::AudioReactor { .. } => ("Peak Value", NodeDataType::Scalar),
            DarkNode::ActionNode { .. } => ("Flow Out", NodeDataType::ActionFlow),
            DarkNode::Reroute { data_type } => ("Out", *data_type),
            _ => ("Out", NodeDataType::Any),
        };

        let resp = ui.label(name);
        if resp.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        PinInfo::circle().with_fill(crate::types::pin_color32(data_type))
    }

    fn has_body(&mut self, node: &DarkNode) -> bool {
        !matches!(node, DarkNode::Reroute { .. })
    }

    fn show_body(
        &mut self,
        node_id: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<DarkNode>,
    ) {
        let node = &mut snarl[node_id];
        crate::node_controls::render_node_body(node, ui);
    }
}
