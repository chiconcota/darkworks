//! Định nghĩa toàn bộ 18 Node Nguyên Tử Đa Năng trong hệ sinh thái Darkworks Node Graph.

use serde::{Deserialize, Serialize};
use crate::node_fx_types::{
    default_audio_track, default_blur_threshold, default_distort_feather, default_keyer_desaturate,
    default_keyer_highlight, default_keyer_luma_cutoff, default_keyer_luma_threshold,
    default_keyer_pedestal, default_keyer_shadow, default_keyer_spill_luma,
    default_neutral_split_tint, default_one, default_stroke_color, default_time_vec2_axis,
    default_tint_color, default_true,
};
use crate::types::NodeDataType;

pub use crate::node_fx_types::{
    ActionScope, AudioBand, BlendMode, BlurDimensions, BlurType, DistortType, KeyerMode,
    KeyerOutput, ShapeType, TextAlign, TimeMode, TransitionDirection, TransitionType,
};

/// Enum đại diện cho mọi Node trên Canvas Darkworks (Hệ sinh thái 18 Atomic Nodes + Final Output).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DarkNode {
    // === NHÓM 1: NGUỒN PHÁT (GENERATORS - 4 NODES) ===
    VideoInput {
        track_name: String,
    },
    TextGenerator {
        text: String,
        font_size: f32,
        color: [f32; 4],
        tracking: f32,
        line_spacing: f32,
        #[serde(default)]
        position: [f32; 2],
        #[serde(default)]
        align: TextAlign,
        #[serde(default)]
        font_family: String,
        #[serde(default)]
        bold: bool,
        #[serde(default)]
        italic: bool,
        #[serde(default = "default_stroke_color")]
        stroke_color: [f32; 4],
        #[serde(default)]
        stroke_width: f32,
    },
    ShapeGenerator {
        shape_type: ShapeType,
        size: [f32; 2],
        corner_radius: f32,
        color: [f32; 4],
        #[serde(default)]
        position: [f32; 2],
        #[serde(default)]
        stroke_width: f32,
        #[serde(default = "default_tint_color")]
        stroke_color: [f32; 4],
        #[serde(default)]
        feather: f32,
    },
    TimeDriver {
        speed: f32,
        mode: TimeMode,
        #[serde(default = "default_one")]
        amplitude: f32,
        #[serde(default)]
        offset: f32,
        #[serde(default = "default_time_vec2_axis")]
        vec2_axis: [f32; 2],
    },

    // === NHÓM 2: CHỈNH MÀU & QUANG HỌC (COLOR & OPTICS - 4 NODES) ===
    ColorGrade {
        exposure: f32,
        contrast: f32,
        saturation: f32,
        temperature: f32,
        tint: f32,
        gamma: f32,
        invert: bool,
        black_and_white: bool,
        #[serde(default = "default_tint_color")]
        tint_color: [f32; 4],
        #[serde(default)]
        tint_amount: f32,
        #[serde(default)]
        highlights: f32,
        #[serde(default)]
        shadows: f32,
        #[serde(default)]
        whites: f32,
        #[serde(default)]
        blacks: f32,
        #[serde(default)]
        vibrance: f32,
        #[serde(default)]
        faded_film: f32,
        #[serde(default = "default_neutral_split_tint")]
        shadow_tint: [f32; 3],
        #[serde(default = "default_neutral_split_tint")]
        highlight_tint: [f32; 3],
    },
    BlurSharpen {
        blur_type: BlurType,
        radius: f32,
        angle_deg: f32,
        highlight_boost: f32,
        #[serde(default = "default_blur_threshold")]
        threshold: f32,
        #[serde(default)]
        dimensions: BlurDimensions,
        #[serde(default = "default_true")]
        repeat_edge: bool,
        #[serde(default)]
        bokeh_sides: u8,
        #[serde(default)]
        center: [f32; 2],
    },
    DistortLens {
        distort_type: DistortType,
        intensity: f32,
        radius: f32,
        center: [f32; 2],
        #[serde(default = "default_distort_feather")]
        feather: f32,
    },
    Keyer {
        key_color: [f32; 3],
        similarity: f32,
        smoothness: f32,
        spill_suppress: f32,
        #[serde(default)]
        mode: KeyerMode,
        #[serde(default)]
        output: KeyerOutput,
        #[serde(default = "default_keyer_pedestal")]
        pedestal: f32,
        #[serde(default = "default_keyer_highlight")]
        highlight: f32,
        #[serde(default = "default_keyer_shadow")]
        shadow: f32,
        #[serde(default)]
        choke: f32,
        #[serde(default)]
        soften: f32,
        #[serde(default)]
        matte_contrast: f32,
        #[serde(default)]
        invert_matte: bool,
        #[serde(default = "default_keyer_desaturate")]
        desaturate: f32,
        #[serde(default = "default_keyer_spill_luma")]
        spill_luma: f32,
        #[serde(default = "default_keyer_luma_threshold")]
        luma_threshold: f32,
        #[serde(default = "default_keyer_luma_cutoff")]
        luma_cutoff: f32,
    },

    // === NHÓM 3: BIẾN HÌNH & HÒA TRỘN (TRANSFORM & COMPOSITING - 3 NODES) ===
    Transform2D {
        position: [f32; 2],
        scale: [f32; 2],
        rotation_deg: f32,
        anchor: [f32; 2],
        opacity: f32,
        flip_h: bool,
        flip_v: bool,
        #[serde(default)]
        uniform_scale: bool,
        #[serde(default)]
        skew: f32,
        #[serde(default)]
        skew_axis: f32,
    },
    CropMask {
        top: f32,
        bottom: f32,
        left: f32,
        right: f32,
        feather: f32,
    },
    Blend {
        mode: BlendMode,
        opacity: f32,
    },

    // === NHÓM 4: CHUYỂN CẢNH (TRANSITIONS - 1 NODE) ===
    Transition {
        transition_type: TransitionType,
        progress: f32,
        feather: f32,
        #[serde(default)]
        direction: TransitionDirection,
    },

    // === NHÓM 5: TÍN HIỆU & TỰ ĐỘNG HÓA (SIGNALS & AUTOMATION - 2 NODES) ===
    AudioReactor {
        band: AudioBand,
        multiplier: f32,
        smooth: f32,
        #[serde(default)]
        vec2_axis: [f32; 2],
        #[serde(default = "default_audio_track")]
        track_name: String,
    },
    ActionNode {
        name: String,
        scope: ActionScope,
        is_running: bool,
    },

    // === NHÓM 6: MỞ RỘNG & TIỆN ÍCH COMFYUI (ADVANCED & ERGONOMICS - 4 NODES) ===
    BlackNode {
        name: String,
        subgraph_id: String,
        exposed_pins: Vec<String>,
    },
    CustomWgsl {
        name: String,
        code: String,
    },
    StickyNote {
        title: String,
        content: String,
        color_idx: usize,
    },
    Reroute {
        data_type: NodeDataType,
    },

    // === KẾT XUẤT CUỐI CÙNG (OUTPUT) ===
    Output,
}

impl DarkNode {
    /// Khởi tạo `TextGenerator` mặc định.
    #[must_use]
    pub fn default_text(text: impl Into<String>) -> Self {
        Self::TextGenerator {
            text: text.into(),
            font_size: 36.0,
            color: [1.0, 1.0, 1.0, 1.0],
            tracking: 0.0,
            line_spacing: 1.1,
            position: [0.0, 0.0],
            align: TextAlign::Center,
            font_family: String::new(),
            bold: false,
            italic: false,
            stroke_color: [0.0, 0.0, 0.0, 1.0],
            stroke_width: 0.0,
        }
    }

    /// Khởi tạo `ShapeGenerator` mặc định.
    #[must_use]
    pub fn default_shape(shape_type: ShapeType, size: [f32; 2], corner_radius: f32, color: [f32; 4]) -> Self {
        Self::ShapeGenerator {
            shape_type,
            size,
            corner_radius,
            color,
            position: [0.0, 0.0],
            stroke_width: 0.0,
            stroke_color: [1.0, 1.0, 1.0, 1.0],
            feather: 0.0,
        }
    }

    /// Khởi tạo `TimeDriver` mặc định.
    #[must_use]
    pub fn default_time_driver(speed: f32, mode: TimeMode) -> Self {
        Self::TimeDriver {
            speed,
            mode,
            amplitude: 1.0,
            offset: 0.0,
            vec2_axis: [60.0, 0.0],
        }
    }

    /// Khởi tạo `ColorGrade` mặc định (tất cả thông số trung tính).
    #[must_use]
    pub fn default_color_grade() -> Self {
        Self::ColorGrade {
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
            highlights: 0.0,
            shadows: 0.0,
            whites: 0.0,
            blacks: 0.0,
            vibrance: 0.0,
            faded_film: 0.0,
            shadow_tint: [0.5, 0.5, 0.5],
            highlight_tint: [0.5, 0.5, 0.5],
        }
    }

    /// Khởi tạo `BlurSharpen` mặc định.
    #[must_use]
    pub fn default_blur(blur_type: BlurType, radius: f32, angle_deg: f32, highlight_boost: f32) -> Self {
        Self::BlurSharpen {
            blur_type,
            radius,
            angle_deg,
            highlight_boost,
            threshold: 0.5,
            dimensions: BlurDimensions::Both,
            repeat_edge: true,
            bokeh_sides: 0,
            center: [0.0, 0.0],
        }
    }

    /// Khởi tạo `DistortLens` mặc định.
    #[must_use]
    pub fn default_distort(distort_type: DistortType, intensity: f32, radius: f32, center: [f32; 2]) -> Self {
        Self::DistortLens {
            distort_type,
            intensity,
            radius,
            center,
            feather: 0.4,
        }
    }

    /// Khởi tạo `Transition` mặc định.
    #[must_use]
    pub fn default_transition(transition_type: TransitionType, progress: f32, feather: f32) -> Self {
        Self::Transition {
            transition_type,
            progress,
            feather,
            direction: TransitionDirection::Left,
        }
    }

    /// Khởi tạo `AudioReactor` mặc định.
    #[must_use]
    pub fn default_audio_reactor(band: AudioBand, multiplier: f32, smooth: f32) -> Self {
        Self::AudioReactor {
            band,
            multiplier,
            smooth,
            vec2_axis: [0.0, 0.0],
            track_name: default_audio_track(),
        }
    }

    /// Khởi tạo `Keyer` (`Ultra Keyer`) mặc định chuẩn Pro NLE.
    #[must_use]
    pub fn default_keyer() -> Self {
        Self::Keyer {
            key_color: [0.0, 1.0, 0.0],
            similarity: 0.4,
            smoothness: 0.1,
            spill_suppress: 0.5,
            mode: KeyerMode::ChromaKey,
            output: KeyerOutput::Composite,
            pedestal: default_keyer_pedestal(),
            highlight: default_keyer_highlight(),
            shadow: default_keyer_shadow(),
            choke: 0.0,
            soften: 0.0,
            matte_contrast: 0.0,
            invert_matte: false,
            desaturate: default_keyer_desaturate(),
            spill_luma: default_keyer_spill_luma(),
            luma_threshold: default_keyer_luma_threshold(),
            luma_cutoff: default_keyer_luma_cutoff(),
        }
    }

    /// Khởi tạo `Transform2D` mặc định (khóa `Uniform Scale` theo chuẩn dựng phim).
    #[must_use]
    pub fn default_transform() -> Self {
        Self::Transform2D {
            position: [0.0, 0.0],
            scale: [1.0, 1.0],
            rotation_deg: 0.0,
            anchor: [0.5, 0.5],
            opacity: 1.0,
            flip_h: false,
            flip_v: false,
            uniform_scale: true,
            skew: 0.0,
            skew_axis: 0.0,
        }
    }

    /// Tiêu đề hiển thị trên thanh Header của Node Card.
    #[must_use]
    pub fn title(&self) -> String {
        match self {
            Self::VideoInput { track_name } => format!("🎬 Video Input ({track_name})"),
            Self::TextGenerator { .. } => "🔤 Text Generator".to_string(),
            Self::ShapeGenerator { shape_type, .. } => format!("🟩 Shape ({shape_type:?})"),
            Self::TimeDriver { .. } => "⏱️ Time Driver".to_string(),

            Self::ColorGrade { invert, black_and_white, .. } => {
                if *invert {
                    "🎨 Color Grade (Invert)".to_string()
                } else if *black_and_white {
                    "🎨 Color Grade (B&W)".to_string()
                } else {
                    "🎨 Color Grade".to_string()
                }
            }
            Self::BlurSharpen { blur_type, .. } => format!("💧 Blur / Sharpen ({blur_type:?})"),
            Self::DistortLens { distort_type, .. } => format!("🌀 Distort / Lens ({distort_type:?})"),
            Self::Keyer { mode, .. } => match mode {
                KeyerMode::ChromaKey => "🟢 Ultra Keyer".to_string(),
                KeyerMode::LumaKey => "◐ Luma Keyer".to_string(),
            },

            Self::Transform2D { .. } => "📐 Transform 2D".to_string(),
            Self::CropMask { .. } => "✂️ Crop / Mask".to_string(),
            Self::Blend { mode, .. } => format!("🎭 Blend ({mode:?})"),
            Self::Transition { transition_type, .. } => format!("⚡ Transition ({transition_type:?})"),

            Self::AudioReactor { band, track_name, .. } => {
                format!("🔊 Audio Reactor ({track_name} · {band:?})")
            }
            Self::ActionNode { name, .. } => format!("⚡ Action: {name}"),

            Self::BlackNode { name, .. } => format!("⬛ Black Box: {name}"),
            Self::CustomWgsl { name, .. } => format!("💻 WGSL: {name}"),
            Self::StickyNote { title, .. } => format!("📝 {title}"),
            Self::Reroute { .. } => "●".to_string(),
            Self::Output => "🖥️ Final Output".to_string(),
        }
    }

    /// Kiểu dữ liệu mong đợi của cổng Input tại vị trí pin.
    #[must_use]
    pub fn input_data_type(&self, pin: usize) -> NodeDataType {
        match self {
            Self::TextGenerator { .. } | Self::ShapeGenerator { .. } => NodeDataType::Vec2,
            Self::Transform2D { .. } => match pin {
                0 => NodeDataType::Texture,
                1 => NodeDataType::Vec2,
                2..=4 => NodeDataType::Scalar,
                _ => NodeDataType::Any,
            },
            Self::ColorGrade { .. } | Self::BlurSharpen { .. } => match pin {
                0 => NodeDataType::Texture,
                _ => NodeDataType::Scalar,
            },
            Self::CropMask { .. } | Self::Keyer { .. } | Self::Output => NodeDataType::Texture,

            Self::DistortLens { .. } => match pin {
                0 => NodeDataType::Texture,
                1 => NodeDataType::Scalar, // Intensity Mod
                _ => NodeDataType::Vec2,   // Center Mod
            },

            Self::Blend { .. } => match pin {
                0 | 1 => NodeDataType::Texture,
                _ => NodeDataType::Texture, // Mask
            },

            Self::Transition { .. } => match pin {
                0 | 1 => NodeDataType::Texture, // Input A & Input B
                _ => NodeDataType::Scalar,      // Progress Mod
            },

            Self::AudioReactor { .. } => NodeDataType::AudioFloat,
            Self::ActionNode { .. } => NodeDataType::ActionFlow,
            Self::CustomWgsl { .. } => match pin {
                0 => NodeDataType::Texture,
                _ => NodeDataType::Scalar,
            },
            Self::Reroute { data_type } => *data_type,
            _ => NodeDataType::Any,
        }
    }

    /// Kiểu dữ liệu xuất ra của cổng Output tại vị trí pin.
    #[must_use]
    pub fn output_data_type(&self, pin: usize) -> NodeDataType {
        match self {
            Self::TimeDriver { .. } => match pin {
                0 => NodeDataType::Scalar,
                _ => NodeDataType::Vec2,
            },
            Self::AudioReactor { .. } => match pin {
                0 => NodeDataType::AudioFloat,
                _ => NodeDataType::Vec2,
            },
            Self::ActionNode { .. } => NodeDataType::ActionFlow,
            Self::Reroute { data_type } => *data_type,
            Self::Output | Self::StickyNote { .. } => NodeDataType::Any,
            _ => NodeDataType::Texture,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyer_legacy_4_field_json_deserializes_with_defaults() {
        let legacy = r#"{"Keyer":{"key_color":[0.0,1.0,0.0],"similarity":0.4,"smoothness":0.2,"spill_suppress":0.8}}"#;
        let node: DarkNode = serde_json::from_str(legacy).expect("legacy Keyer JSON must deserialize");
        match node {
            DarkNode::Keyer {
                key_color,
                similarity,
                smoothness,
                spill_suppress,
                mode,
                output,
                pedestal,
                highlight,
                shadow,
                choke,
                soften,
                matte_contrast,
                invert_matte,
                desaturate,
                spill_luma,
                luma_threshold,
                luma_cutoff,
            } => {
                assert_eq!(key_color, [0.0, 1.0, 0.0]);
                assert!((similarity - 0.4).abs() < 1e-6);
                assert!((smoothness - 0.2).abs() < 1e-6);
                assert!((spill_suppress - 0.8).abs() < 1e-6);
                assert_eq!(mode, KeyerMode::ChromaKey);
                assert_eq!(output, KeyerOutput::Composite);
                assert!((pedestal - 0.10).abs() < 1e-6);
                assert!((highlight - 0.10).abs() < 1e-6);
                assert!((shadow - 0.50).abs() < 1e-6);
                assert_eq!(choke, 0.0);
                assert_eq!(soften, 0.0);
                assert_eq!(matte_contrast, 0.0);
                assert!(!invert_matte);
                assert!((desaturate - 0.25).abs() < 1e-6);
                assert!((spill_luma - 0.50).abs() < 1e-6);
                assert!((luma_threshold - 0.15).abs() < 1e-6);
                assert!((luma_cutoff - 0.10).abs() < 1e-6);
            }
            other => panic!("unexpected node: {other:?}"),
        }
    }
}
