//! Các module xử lý pixel chuyên biệt cho từng nhóm Atomic Node trong `crates/render`.

pub(crate) mod blur_distort;
pub(crate) mod color;
pub(crate) mod gpu_plan;
pub(crate) mod shape;
pub(crate) mod signals;
pub(crate) mod text;
pub(crate) mod transform_blend;

pub(crate) use blur_distort::{apply_blur_sharpen, apply_distort};
pub(crate) use color::{apply_color_grade, apply_keyer};
pub(crate) use gpu_plan::{eval_graph_to_fx_ops, graph_needs_below, graph_replaces_below};
pub(crate) use shape::render_shape;
pub(crate) use signals::{
    compute_audio_rms_map, count_wire_uses, modulated_scalar, modulated_scalar_add,
    modulated_scalar_add_scaled, modulated_scalar_mul, modulated_scalar_override,
    modulated_vec2, modulated_vec2_add, resolve_video_input, take_or_clone_texture,
};
pub(crate) use text::render_text;
pub(crate) use transform_blend::{apply_mask_alpha, apply_transform, apply_transition, map_blend};
