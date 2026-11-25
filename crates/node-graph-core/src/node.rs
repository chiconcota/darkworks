//! Định nghĩa toàn bộ 18 Node Nguyên Tử Đa Năng trong hệ sinh thái Darkworks Node Graph.

use serde::{Deserialize, Serialize};
use crate::types::NodeDataType;

/// Kiểu hình khối cơ bản của `ShapeGenerator`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapeType {
    Rectangle,
    RoundedRectangle,
    Circle,
    SolidMatte,
}

/// Chế độ đếm thời gian của `TimeDriver`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeMode {
    Linear,
    SineLoop,
    PingPong,
}

/// 27 chế độ hòa trộn của `Blend` (khớp 100% với `filmcraft_render::Blend`, kèm `Add` tương thích ngược)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Dissolve,
    Darken,
    Multiply,
    ColorBurn,
    LinearBurn,
    DarkerColor,
    Lighten,
    Screen,
    ColorDodge,
    LinearDodge,
    LighterColor,
    Overlay,
    SoftLight,
    HardLight,
    VividLight,
    LinearLight,
    PinLight,
    HardMix,
    Difference,
    Exclusion,
    Subtract,
    Divide,
    Hue,
    Saturation,
    Color,
    Luminosity,
    Add,
}

impl BlendMode {
    /// Toàn bộ các chế độ hòa trộn hiển thị trong menu chọn của node `Blend`.
    pub const ALL: &'static [Self] = &[
        Self::Normal,
        Self::Dissolve,
        Self::Darken,
        Self::Multiply,
        Self::ColorBurn,
        Self::LinearBurn,
        Self::DarkerColor,
        Self::Lighten,
        Self::Screen,
        Self::ColorDodge,
        Self::LinearDodge,
        Self::LighterColor,
        Self::Overlay,
        Self::SoftLight,
        Self::HardLight,
        Self::VividLight,
        Self::LinearLight,
        Self::PinLight,
        Self::HardMix,
        Self::Difference,
        Self::Exclusion,
        Self::Subtract,
        Self::Divide,
        Self::Hue,
        Self::Saturation,
        Self::Color,
        Self::Luminosity,
        Self::Add,
    ];

    /// Nhãn hiển thị thân thiện trên UI.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Dissolve => "Dissolve",
            Self::Darken => "Darken",
            Self::Multiply => "Multiply",
            Self::ColorBurn => "Color Burn",
            Self::LinearBurn => "Linear Burn",
            Self::DarkerColor => "Darker Color",
            Self::Lighten => "Lighten",
            Self::Screen => "Screen",
            Self::ColorDodge => "Color Dodge",
            Self::LinearDodge => "Linear Dodge",
            Self::LighterColor => "Lighter Color",
            Self::Overlay => "Overlay",
            Self::SoftLight => "Soft Light",
            Self::HardLight => "Hard Light",
            Self::VividLight => "Vivid Light",
            Self::LinearLight => "Linear Light",
            Self::PinLight => "Pin Light",
            Self::HardMix => "Hard Mix",
            Self::Difference => "Difference",
            Self::Exclusion => "Exclusion",
            Self::Subtract => "Subtract",
            Self::Divide => "Divide",
            Self::Hue => "Hue",
            Self::Saturation => "Saturation",
            Self::Color => "Color",
            Self::Luminosity => "Luminosity",
            Self::Add => "Add (Linear Dodge)",
        }
    }
}

/// Dải tần số âm thanh cho `AudioReactor`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioBand {
    SubBass,
    Bass,
    LowMid,
    HighMid,
    Treble,
    MasterPeak,
}

/// Phạm vi tác động của `ActionNode`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionScope {
    MarkedInOut,
    SelectedClips,
    TrackV1,
    TrackV2,
    AllTracks,
}

/// 5 Thuật toán làm mờ và làm nét của `BlurSharpen` (Gom 5 Shader WGSL)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlurType {
    Gaussian,
    Box,
    Directional,
    BokehBloom,
    UnsharpMask,
}

/// 5 Thuật toán biến dạng thấu kính của `DistortLens` (Gom 6 Shader WGSL)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistortType {
    Vignette,
    FishEye,
    LensDistortion,
    RippleWave,
    ChromaticAberration,
}

/// 5 Thuật toán chuyển cảnh của `Transition` (Gom 30 Shader WGSL)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionType {
    CrossDissolve,
    LinearWipe,
    Slide,
    Iris,
    ZoomPush,
}

fn default_tint_color() -> [f32; 4] {
    [1.0, 1.0, 1.0, 1.0]
}

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
    },
    ShapeGenerator {
        shape_type: ShapeType,
        size: [f32; 2],
        corner_radius: f32,
        color: [f32; 4],
    },
    TimeDriver {
        speed: f32,
        mode: TimeMode,
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
    },
    BlurSharpen {
        blur_type: BlurType,
        radius: f32,
        angle_deg: f32,
        highlight_boost: f32,
    },
    DistortLens {
        distort_type: DistortType,
        intensity: f32,
        radius: f32,
        center: [f32; 2],
    },
    Keyer {
        key_color: [f32; 3],
        similarity: f32,
        smoothness: f32,
        spill_suppress: f32,
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
    },

    // === NHÓM 5: TÍN HIỆU & TỰ ĐỘNG HÓA (SIGNALS & AUTOMATION - 2 NODES) ===
    AudioReactor {
        band: AudioBand,
        multiplier: f32,
        smooth: f32,
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
            Self::Keyer { .. } => "🟢 Ultra Keyer".to_string(),

            Self::Transform2D { .. } => "📐 Transform 2D".to_string(),
            Self::CropMask { .. } => "✂️ Crop / Mask".to_string(),
            Self::Blend { mode, .. } => format!("🎭 Blend ({mode:?})"),
            Self::Transition { transition_type, .. } => format!("⚡ Transition ({transition_type:?})"),

            Self::AudioReactor { band, .. } => format!("🔊 Audio Reactor ({band:?})"),
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
            Self::Transform2D { .. } => match pin {
                0 => NodeDataType::Texture,
                1 => NodeDataType::Vec2,
                2..=4 => NodeDataType::Scalar,
                _ => NodeDataType::Any,
            },
            Self::CropMask { .. }
            | Self::ColorGrade { .. }
            | Self::BlurSharpen { .. }
            | Self::Keyer { .. }
            | Self::Output => NodeDataType::Texture,

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
    pub fn output_data_type(&self, _pin: usize) -> NodeDataType {
        match self {
            Self::TimeDriver { .. } => NodeDataType::Scalar,
            Self::AudioReactor { .. } => NodeDataType::AudioFloat,
            Self::ActionNode { .. } => NodeDataType::ActionFlow,
            Self::Reroute { data_type } => *data_type,
            Self::Output | Self::StickyNote { .. } => NodeDataType::Any,
            _ => NodeDataType::Texture,
        }
    }
}
