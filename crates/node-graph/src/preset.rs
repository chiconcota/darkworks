//! Quản lý lưu và nạp cấu trúc đồ thị sang file `.preset.json` và kho 5 Preset Mẫu cốt lõi.

use egui::{Color32, Pos2, Vec2};
use egui_snarl::{InPinId, OutPinId, Snarl};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::group::VisualGroup;
use crate::node::{
    AudioBand, BlendMode, BlurType, DarkNode, DistortType, ShapeType, TimeMode,
};

#[derive(Debug, Error)]
pub enum PresetError {
    #[error("Lỗi tuần tự hóa JSON: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Lỗi I/O tập tin: {0}")]
    IoError(#[from] std::io::Error),
}

/// Loại Preset mẫu dựng sẵn trong hệ thống Darkworks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuiltinPresetKind {
    /// 1. Dải tin tức News Ticker nền xanh chữ đen
    NewsTickerBanner,
    /// 2. Đường sóng âm liên tục theo nhạc 60 FPS (Continuous Waveform Line)
    ContinuousAudioWaveform,
    /// 3. Mặt nạ tròn đảo âm bản (Spotlight Invert)
    SpotlightInvert,
    /// 4. Chớp giật âm bản theo nhịp Bass (Audio Beat Invert Flash)
    AudioBeatFlash,
    /// 5. Phát sáng điện ảnh mềm mại (Cinematic Soft Glow)
    CinematicSoftGlow,
}

/// Cấu trúc lưu trữ hoàn chỉnh của một file preset đồ thị (.preset.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGraphPreset {
    pub name: String,
    pub version: String,
    pub description: String,
    pub nodes: Vec<DarkNode>,
    pub groups: Vec<VisualGroup>,
}

impl NodeGraphPreset {
    /// Xuất đồ thị sang chuỗi JSON đẹp (Pretty JSON).
    pub fn to_json_string(&self) -> Result<String, PresetError> {
        serde_json::to_string_pretty(self).map_err(PresetError::JsonError)
    }

    /// Nạp cấu trúc đồ thị từ chuỗi JSON.
    pub fn from_json_str(json: &str) -> Result<Self, PresetError> {
        serde_json::from_str(json).map_err(PresetError::JsonError)
    }

    /// Tạo Preset từ đồ thị Snarl hiện có.
    #[must_use]
    pub fn from_snarl(name: impl Into<String>, snarl: &Snarl<DarkNode>) -> Self {
        let nodes = snarl.node_ids().map(|(_id, node)| node.clone()).collect();
        Self {
            name: name.into(),
            version: "1.0.0".to_string(),
            description: "Exported Node Graph Preset".to_string(),
            nodes,
            groups: Vec::new(),
        }
    }

    /// Áp dụng các Node từ Preset vào Snarl.
    pub fn apply_to_snarl(&self, snarl: &mut Snarl<DarkNode>) {
        for (i, node) in self.nodes.iter().enumerate() {
            snarl.insert_node(
                egui::Pos2::new(100.0 + (i as f32) * 60.0, 100.0 + (i as f32) * 60.0),
                node.clone(),
            );
        }
    }

    /// Khởi tạo Preset 1: Dải tin tức News Ticker nền xanh chữ đen
    #[must_use]
    pub fn build_news_ticker_preset() -> (Snarl<DarkNode>, Vec<VisualGroup>) {
        let mut snarl = Snarl::<DarkNode>::new();

        let shape_id = snarl.insert_node(
            Pos2::new(80.0, 100.0),
            DarkNode::default_shape(
                ShapeType::RoundedRectangle,
                [1920.0, 80.0],
                8.0,
                [0.0, 0.9, 0.4, 1.0], // Neon Green
            ),
        );

        let text_id = snarl.insert_node(
            Pos2::new(80.0, 260.0),
            DarkNode::TextGenerator {
                text: "BẢN TIN NÓNG: DARKWORKS NODE GRAPH ĐÃ KHỞI CHẠY!".to_string(),
                font_size: 26.0,
                color: [0.0, 0.0, 0.0, 1.0], // Black text
                tracking: 1.0,
                line_spacing: 1.2,
                position: [0.0, 0.0],
                align: Default::default(),
                font_family: String::new(),
                bold: true,
                italic: false,
                stroke_color: [0.0, 0.0, 0.0, 1.0],
                stroke_width: 0.0,
            },
        );

        let time_id = snarl.insert_node(
            Pos2::new(80.0, 420.0),
            DarkNode::default_time_driver(2.0, TimeMode::Linear),
        );

        let blend_id = snarl.insert_node(
            Pos2::new(420.0, 180.0),
            DarkNode::Blend {
                mode: BlendMode::Normal,
                opacity: 1.0,
            },
        );

        let output_id = snarl.insert_node(Pos2::new(720.0, 180.0), DarkNode::Output);

        // Nối dây
        snarl.connect(
            OutPinId { node: shape_id, output: 0 },
            InPinId { node: blend_id, input: 0 },
        );
        snarl.connect(
            OutPinId { node: text_id, output: 0 },
            InPinId { node: blend_id, input: 1 },
        );
        snarl.connect(
            OutPinId { node: blend_id, output: 0 },
            InPinId { node: output_id, input: 0 },
        );
        let _ = time_id;

        let group = VisualGroup::new(
            1,
            "1. NEWS TICKER BANNER PIPELINE".to_string(),
            Pos2::new(60.0, 60.0),
            Vec2::new(600.0, 440.0),
            Color32::from_rgba_unmultiplied(0, 180, 255, 30),
        );

        (snarl, vec![group])
    }

    /// Khởi tạo Preset 2: Đường sóng âm liên tục theo nhạc 60 FPS (Continuous Waveform Line)
    #[must_use]
    pub fn build_audio_waveform_preset() -> (Snarl<DarkNode>, Vec<VisualGroup>) {
        let mut snarl = Snarl::<DarkNode>::new();

        // 1. Sinh đường thẳng ngang 4px
        let line_id = snarl.insert_node(
            Pos2::new(80.0, 80.0),
            DarkNode::default_shape(
                ShapeType::Rectangle,
                [1920.0, 4.0],
                0.0,
                [0.0, 0.9, 1.0, 1.0], // Neon Cyan Line
            ),
        );

        // 2. AudioReactor bắt nhịp Master Peak
        let audio_id = snarl.insert_node(
            Pos2::new(80.0, 240.0),
            DarkNode::default_audio_reactor(AudioBand::MasterPeak, 2.0, 0.3),
        );

        // 3. TimeDriver cuộn sóng chạy liên tục
        let time_id = snarl.insert_node(
            Pos2::new(80.0, 380.0),
            DarkNode::default_time_driver(3.0, TimeMode::Linear),
        );

        // 4. DistortLens (Ripple / Wave) biến đường thẳng thành sóng sin uốn lượn
        let distort_id = snarl.insert_node(
            Pos2::new(380.0, 120.0),
            DarkNode::default_distort(DistortType::RippleWave, 0.8, 1.0, [0.0, 0.0]),
        );

        // 5. Video Input (Phim nền)
        let video_in_id = snarl.insert_node(
            Pos2::new(380.0, 320.0),
            DarkNode::VideoInput { track_name: "Auto".to_string() },
        );

        // 6. Blend (Screen hòa sóng nhạc phát sáng lên nền video)
        let blend_id = snarl.insert_node(
            Pos2::new(680.0, 200.0),
            DarkNode::Blend {
                mode: BlendMode::Screen,
                opacity: 1.0,
            },
        );

        // 7. Output
        let output_id = snarl.insert_node(Pos2::new(960.0, 200.0), DarkNode::Output);

        // Nối dây:
        // Shape Line -> Distort Image In (pin 0)
        snarl.connect(
            OutPinId { node: line_id, output: 0 },
            InPinId { node: distort_id, input: 0 },
        );
        // AudioReactor -> Distort Intensity Mod (pin 1)
        snarl.connect(
            OutPinId { node: audio_id, output: 0 },
            InPinId { node: distort_id, input: 1 },
        );
        // Video Input -> Blend Background (pin 0)
        snarl.connect(
            OutPinId { node: video_in_id, output: 0 },
            InPinId { node: blend_id, input: 0 },
        );
        // Distort Wave -> Blend Foreground (pin 1)
        snarl.connect(
            OutPinId { node: distort_id, output: 0 },
            InPinId { node: blend_id, input: 1 },
        );
        // Blend -> Output
        snarl.connect(
            OutPinId { node: blend_id, output: 0 },
            InPinId { node: output_id, input: 0 },
        );
        let _ = time_id;

        let group = VisualGroup::new(
            2,
            "2. CONTINUOUS AUDIO WAVEFORM LINE PIPELINE".to_string(),
            Pos2::new(60.0, 40.0),
            Vec2::new(850.0, 440.0),
            Color32::from_rgba_unmultiplied(0, 230, 118, 30),
        );

        (snarl, vec![group])
    }

    /// Khởi tạo Preset 3: Mặt nạ tròn đảo âm bản (Spotlight Invert)
    #[must_use]
    pub fn build_spotlight_invert_preset() -> (Snarl<DarkNode>, Vec<VisualGroup>) {
        let mut snarl = Snarl::<DarkNode>::new();

        let video_in_id = snarl.insert_node(
            Pos2::new(80.0, 80.0),
            DarkNode::VideoInput { track_name: "Auto".to_string() },
        );

        let solid_white_id = snarl.insert_node(
            Pos2::new(80.0, 220.0),
            DarkNode::default_shape(
                ShapeType::SolidMatte,
                [1920.0, 1080.0],
                0.0,
                [1.0, 1.0, 1.0, 1.0], // Pure White
            ),
        );

        let circle_mask_id = snarl.insert_node(
            Pos2::new(80.0, 360.0),
            DarkNode::ShapeGenerator {
                shape_type: ShapeType::Circle,
                size: [400.0, 400.0],
                corner_radius: 0.0,
                color: [1.0, 1.0, 1.0, 1.0],
                position: [0.0, 0.0],
                stroke_width: 0.0,
                stroke_color: [1.0, 1.0, 1.0, 1.0],
                feather: 18.0,
            },
        );

        let blend_id = snarl.insert_node(
            Pos2::new(450.0, 180.0),
            DarkNode::Blend {
                mode: BlendMode::Difference,
                opacity: 1.0,
            },
        );

        let output_id = snarl.insert_node(Pos2::new(750.0, 180.0), DarkNode::Output);

        // Nối dây:
        // Video Input -> Blend Background
        snarl.connect(
            OutPinId { node: video_in_id, output: 0 },
            InPinId { node: blend_id, input: 0 },
        );
        // Solid White -> Blend Foreground
        snarl.connect(
            OutPinId { node: solid_white_id, output: 0 },
            InPinId { node: blend_id, input: 1 },
        );
        // Circle Mask -> Blend Mask (pin 2)
        snarl.connect(
            OutPinId { node: circle_mask_id, output: 0 },
            InPinId { node: blend_id, input: 2 },
        );
        // Blend -> Output
        snarl.connect(
            OutPinId { node: blend_id, output: 0 },
            InPinId { node: output_id, input: 0 },
        );

        let group = VisualGroup::new(
            3,
            "3. SPOTLIGHT INVERT PIPELINE".to_string(),
            Pos2::new(60.0, 40.0),
            Vec2::new(650.0, 420.0),
            Color32::from_rgba_unmultiplied(255, 64, 129, 30),
        );

        (snarl, vec![group])
    }

    /// Khởi tạo Preset 4: Chớp sáng theo nhịp Bass (Audio Beat Flash)
    #[must_use]
    pub fn build_audio_beat_flash_preset() -> (Snarl<DarkNode>, Vec<VisualGroup>) {
        let mut snarl = Snarl::<DarkNode>::new();

        let video_in_id = snarl.insert_node(
            Pos2::new(80.0, 100.0),
            DarkNode::VideoInput { track_name: "Auto".to_string() },
        );

        let audio_id = snarl.insert_node(
            Pos2::new(80.0, 260.0),
            DarkNode::default_audio_reactor(AudioBand::Bass, 1.4, 0.15),
        );

        let grade_id = snarl.insert_node(
            Pos2::new(420.0, 140.0),
            DarkNode::default_color_grade(),
        );

        let output_id = snarl.insert_node(Pos2::new(740.0, 140.0), DarkNode::Output);

        // Nối dây: Video -> ColorGrade (pin 0), AudioReactor -> ColorGrade Exp Mod (pin 1) -> Output
        snarl.connect(
            OutPinId { node: video_in_id, output: 0 },
            InPinId { node: grade_id, input: 0 },
        );
        snarl.connect(
            OutPinId { node: audio_id, output: 0 },
            InPinId { node: grade_id, input: 1 },
        );
        snarl.connect(
            OutPinId { node: grade_id, output: 0 },
            InPinId { node: output_id, input: 0 },
        );

        let group = VisualGroup::new(
            4,
            "4. AUDIO BEAT FLASH PIPELINE".to_string(),
            Pos2::new(60.0, 50.0),
            Vec2::new(640.0, 380.0),
            Color32::from_rgba_unmultiplied(255, 214, 0, 30),
        );

        (snarl, vec![group])
    }

    /// Khởi tạo Preset 5: Phát sáng điện ảnh mềm mại (Cinematic Soft Glow)
    #[must_use]
    pub fn build_cinematic_glow_preset() -> (Snarl<DarkNode>, Vec<VisualGroup>) {
        let mut snarl = Snarl::<DarkNode>::new();

        let video_in_id = snarl.insert_node(
            Pos2::new(80.0, 120.0),
            DarkNode::VideoInput { track_name: "Auto".to_string() },
        );

        let blur_id = snarl.insert_node(
            Pos2::new(380.0, 240.0),
            DarkNode::default_blur(BlurType::BokehBloom, 25.0, 0.0, 1.5),
        );

        let blend_id = snarl.insert_node(
            Pos2::new(680.0, 160.0),
            DarkNode::Blend {
                mode: BlendMode::Screen,
                opacity: 0.75,
            },
        );

        let output_id = snarl.insert_node(Pos2::new(960.0, 160.0), DarkNode::Output);

        // Nối dây:
        // Video Input -> Blend Background (pin 0)
        snarl.connect(
            OutPinId { node: video_in_id, output: 0 },
            InPinId { node: blend_id, input: 0 },
        );
        // Video Input -> Blur Bokeh Bloom (pin 0)
        snarl.connect(
            OutPinId { node: video_in_id, output: 0 },
            InPinId { node: blur_id, input: 0 },
        );
        // Blur Bokeh -> Blend Foreground (pin 1)
        snarl.connect(
            OutPinId { node: blur_id, output: 0 },
            InPinId { node: blend_id, input: 1 },
        );
        // Blend -> Output
        snarl.connect(
            OutPinId { node: blend_id, output: 0 },
            InPinId { node: output_id, input: 0 },
        );

        let group = VisualGroup::new(
            5,
            "5. CINEMATIC SOFT GLOW PIPELINE".to_string(),
            Pos2::new(60.0, 60.0),
            Vec2::new(850.0, 350.0),
            Color32::from_rgba_unmultiplied(170, 0, 255, 30),
        );

        (snarl, vec![group])
    }

    /// Nạp nhanh preset theo phân loại `BuiltinPresetKind`.
    #[must_use]
    pub fn load_builtin(kind: BuiltinPresetKind) -> (Snarl<DarkNode>, Vec<VisualGroup>) {
        match kind {
            BuiltinPresetKind::NewsTickerBanner => Self::build_news_ticker_preset(),
            BuiltinPresetKind::ContinuousAudioWaveform => Self::build_audio_waveform_preset(),
            BuiltinPresetKind::SpotlightInvert => Self::build_spotlight_invert_preset(),
            BuiltinPresetKind::AudioBeatFlash => Self::build_audio_beat_flash_preset(),
            BuiltinPresetKind::CinematicSoftGlow => Self::build_cinematic_glow_preset(),
        }
    }

    /// Trả về [`darkworks_node_graph_core::NodeGraphDoc`] của Preset mẫu để hỗ trợ Replace/Chain.
    #[must_use]
    pub fn builtin_doc(kind: BuiltinPresetKind) -> darkworks_node_graph_core::NodeGraphDoc {
        let (snarl, groups) = Self::load_builtin(kind);
        crate::doc::doc_from_snarl(&snarl, &groups, &std::collections::HashMap::new())
    }
}
