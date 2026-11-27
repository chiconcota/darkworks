//! Các kiểu enum và giá trị mặc định phục vụ serialization cho hệ thống Atomic Nodes (`DarkNode`).

use serde::{Deserialize, Serialize};

/// Kiểu hình khối cơ bản của `ShapeGenerator`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapeType {
    Rectangle,
    RoundedRectangle,
    Circle,
    SolidMatte,
}

/// Căn lề văn bản cho `TextGenerator`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    #[default]
    Center,
    Right,
}

/// Hướng chuyển cảnh cho `Transition`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TransitionDirection {
    #[default]
    Left,
    Right,
    Up,
    Down,
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

/// Các thuật toán làm mờ và làm nét của `BlurSharpen`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlurType {
    Gaussian,
    Box,
    Directional,
    BokehBloom,
    UnsharpMask,
    RadialZoom,
}

/// Trục làm mờ cho `BlurSharpen` (`Gaussian` & `Box`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BlurDimensions {
    #[default]
    Both,
    Horizontal,
    Vertical,
}

/// Thuật toán biến dạng & stylize của `DistortLens`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistortType {
    Vignette,
    FishEye,
    LensDistortion,
    RippleWave,
    ChromaticAberration,
    Mirror,
    Offset,
    Posterize,
    Pixelate,
    Twirl,
    TurbulentDisplace,
    FindEdges,
    NoiseGrain,
}

/// Thuật toán chuyển cảnh của `Transition`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionType {
    CrossDissolve,
    LinearWipe,
    Slide,
    Iris,
    ZoomPush,
    DipBlack,
    DipWhite,
    BlurDissolve,
}

/// Chế độ tách phông của `Ultra Keyer`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum KeyerMode {
    #[default]
    ChromaKey,
    LumaKey,
}

/// Chế độ xuất kết quả của `Ultra Keyer`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum KeyerOutput {
    #[default]
    Composite,
    AlphaChannel,
    ColorChannel,
}

pub(crate) fn default_tint_color() -> [f32; 4] {
    [1.0, 1.0, 1.0, 1.0]
}
pub(crate) fn default_neutral_split_tint() -> [f32; 3] {
    [0.5, 0.5, 0.5]
}
pub(crate) fn default_one() -> f32 {
    1.0
}
pub(crate) fn default_true() -> bool {
    true
}
pub(crate) fn default_blur_threshold() -> f32 {
    0.5
}
pub(crate) fn default_distort_feather() -> f32 {
    0.4
}
pub(crate) fn default_time_vec2_axis() -> [f32; 2] {
    [60.0, 0.0]
}
pub(crate) fn default_stroke_color() -> [f32; 4] {
    [0.0, 0.0, 0.0, 1.0]
}
pub(crate) fn default_audio_track() -> String {
    "Master".to_string()
}
pub(crate) fn default_keyer_pedestal() -> f32 {
    0.10
}
pub(crate) fn default_keyer_highlight() -> f32 {
    0.10
}
pub(crate) fn default_keyer_shadow() -> f32 {
    0.50
}
pub(crate) fn default_keyer_desaturate() -> f32 {
    0.25
}
pub(crate) fn default_keyer_spill_luma() -> f32 {
    0.50
}
pub(crate) fn default_keyer_luma_threshold() -> f32 {
    0.15
}
pub(crate) fn default_keyer_luma_cutoff() -> f32 {
    0.10
}
