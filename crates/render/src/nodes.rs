//! Các module xử lý pixel chuyên biệt cho từng nhóm Atomic Node trong `crates/render`.

pub(crate) mod blur_distort;
pub(crate) mod color;
pub(crate) mod shape;
pub(crate) mod text;
pub(crate) mod transform_blend;

pub(crate) use blur_distort::{apply_blur_sharpen, apply_distort};
pub(crate) use color::{apply_color_grade, apply_keyer};
pub(crate) use shape::render_shape;
pub(crate) use text::render_text;
pub(crate) use transform_blend::{apply_mask_alpha, apply_transform, apply_transition, map_blend};
