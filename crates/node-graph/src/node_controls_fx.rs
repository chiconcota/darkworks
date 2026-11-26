//! Bộ điều khiển UI cho nhóm Node Chỉnh màu, Quang học & Chuyển cảnh
//! (`ColorGrade`, `BlurSharpen`, `DistortLens`, `Transition`, `Keyer`).

use egui::{ComboBox, DragValue, Grid, Ui};

use crate::node::{
    BlurDimensions, BlurType, DarkNode, DistortType, TransitionDirection, TransitionType,
};
use crate::node_controls::{check_box, color_picker_rgba, drag_val};

/// Render UI controls cho nhóm hiệu ứng hình ảnh; trả về `true` nếu node thuộc nhóm này.
pub(crate) fn render_fx_controls(node: &mut DarkNode, node_ptr: usize, ui: &mut Ui) -> bool {
    match node {
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
            highlights,
            shadows,
            whites,
            blacks,
            vibrance,
            faded_film,
            shadow_tint,
            highlight_tint,
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
            ui.collapsing("🎛 Tonal & Split Tone", |ui| {
                Grid::new(("cg_lum_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label("High:");
                    drag_val(ui, DragValue::new(highlights).range(-100.0..=100.0).speed(0.5));
                    ui.end_row();
                    ui.label("Shad:");
                    drag_val(ui, DragValue::new(shadows).range(-100.0..=100.0).speed(0.5));
                    ui.end_row();
                    ui.label("Whites:");
                    drag_val(ui, DragValue::new(whites).range(-100.0..=100.0).speed(0.5));
                    ui.end_row();
                    ui.label("Blacks:");
                    drag_val(ui, DragValue::new(blacks).range(-100.0..=100.0).speed(0.5));
                    ui.end_row();
                    ui.label("Vibrance:");
                    drag_val(ui, DragValue::new(vibrance).range(-100.0..=100.0).speed(0.5));
                    ui.end_row();
                    ui.label("Faded:");
                    drag_val(ui, DragValue::new(faded_film).range(0.0..=100.0).speed(0.5));
                    ui.end_row();
                    ui.label("Shad Tint:");
                    ui.color_edit_button_rgb(shadow_tint);
                    ui.end_row();
                    ui.label("High Tint:");
                    ui.color_edit_button_rgb(highlight_tint);
                    ui.end_row();
                });
            });
            check_box(ui, invert, "⚡ Invert Colors");
            check_box(ui, black_and_white, "⬛ B&W");
            true
        }
        DarkNode::BlurSharpen {
            blur_type,
            radius,
            angle_deg,
            highlight_boost,
            threshold,
            dimensions,
            repeat_edge,
            bokeh_sides,
            center,
        } => {
            ComboBox::from_id_salt(("blur_type", node_ptr))
                .selected_text(format!("{blur_type:?}"))
                .show_ui(ui, |ui| {
                    ui.selectable_value(blur_type, BlurType::Gaussian, "Gaussian");
                    ui.selectable_value(blur_type, BlurType::Box, "Box");
                    ui.selectable_value(blur_type, BlurType::Directional, "Directional");
                    ui.selectable_value(blur_type, BlurType::BokehBloom, "Bokeh Bloom");
                    ui.selectable_value(blur_type, BlurType::UnsharpMask, "Unsharp Mask");
                    ui.selectable_value(blur_type, BlurType::RadialZoom, "Radial Zoom");
                });
            Grid::new(("blur_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                ui.label("Radius:");
                drag_val(ui, DragValue::new(radius).range(0.0..=150.0).speed(0.5));
                ui.end_row();
                if matches!(blur_type, BlurType::Gaussian | BlurType::Box) {
                    ui.label("Dim:");
                    let dim_label = match dimensions {
                        BlurDimensions::Both => "H + V",
                        BlurDimensions::Horizontal => "Horizontal",
                        BlurDimensions::Vertical => "Vertical",
                    };
                    ComboBox::from_id_salt(("blur_dim", node_ptr))
                        .selected_text(dim_label)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(dimensions, BlurDimensions::Both, "H + V");
                            ui.selectable_value(dimensions, BlurDimensions::Horizontal, "Horizontal");
                            ui.selectable_value(dimensions, BlurDimensions::Vertical, "Vertical");
                        });
                    ui.end_row();
                }
                if matches!(blur_type, BlurType::Directional | BlurType::BokehBloom) {
                    ui.label("Angle:");
                    drag_val(ui, DragValue::new(angle_deg).range(0.0..=360.0).suffix("°"));
                    ui.end_row();
                }
                if matches!(blur_type, BlurType::BokehBloom | BlurType::UnsharpMask) {
                    ui.label(if *blur_type == BlurType::UnsharpMask { "Amount:" } else { "Boost:" });
                    drag_val(ui, DragValue::new(highlight_boost).range(0.0..=5.0).speed(0.05));
                    ui.end_row();
                    ui.label("Thresh:");
                    drag_val(ui, DragValue::new(threshold).range(0.0..=1.0).speed(0.02));
                    ui.end_row();
                }
                if *blur_type == BlurType::BokehBloom {
                    ui.label("Sides:");
                    let mut sides_val = *bokeh_sides;
                    drag_val(ui, DragValue::new(&mut sides_val).range(0..=8));
                    *bokeh_sides = sides_val;
                    ui.end_row();
                }
                if *blur_type == BlurType::RadialZoom {
                    ui.label("Center X:");
                    drag_val(ui, DragValue::new(&mut center[0]).range(-1.5..=1.5).speed(0.02));
                    ui.end_row();
                    ui.label("Center Y:");
                    drag_val(ui, DragValue::new(&mut center[1]).range(-1.5..=1.5).speed(0.02));
                    ui.end_row();
                }
            });
            if matches!(blur_type, BlurType::Gaussian | BlurType::Box) {
                check_box(ui, repeat_edge, "Repeat Edge");
            }
            true
        }
        DarkNode::DistortLens {
            distort_type,
            intensity,
            radius,
            center,
            feather,
        } => {
            ComboBox::from_id_salt(("dist_type", node_ptr))
                .selected_text(format!("{distort_type:?}"))
                .show_ui(ui, |ui| {
                    for (ty, label) in [
                        (DistortType::Vignette, "Vignette"),
                        (DistortType::FishEye, "FishEye"),
                        (DistortType::LensDistortion, "Lens Distortion"),
                        (DistortType::RippleWave, "Ripple / Wave"),
                        (DistortType::Twirl, "Twirl"),
                        (DistortType::TurbulentDisplace, "Turbulent Displace"),
                        (DistortType::ChromaticAberration, "Chromatic Aberration"),
                        (DistortType::Mirror, "Mirror"),
                        (DistortType::Offset, "Offset (Wrap)"),
                        (DistortType::Posterize, "Posterize"),
                        (DistortType::Pixelate, "Pixelate (Mosaic)"),
                        (DistortType::FindEdges, "Find Edges"),
                        (DistortType::NoiseGrain, "Noise / Grain"),
                    ] {
                        ui.selectable_value(distort_type, ty, label);
                    }
                });
            Grid::new(("dist_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                if *distort_type != DistortType::Offset {
                    let int_label = match distort_type {
                        DistortType::Mirror | DistortType::Twirl => "Angle:",
                        DistortType::Posterize => "Levels:",
                        DistortType::NoiseGrain => "Amount:",
                        _ => "Intensity:",
                    };
                    ui.label(int_label);
                    drag_val(ui, DragValue::new(intensity).range(-2.0..=2.0).speed(0.02));
                    ui.end_row();
                }
                if matches!(
                    distort_type,
                    DistortType::Vignette
                        | DistortType::RippleWave
                        | DistortType::Pixelate
                        | DistortType::Twirl
                        | DistortType::TurbulentDisplace
                        | DistortType::NoiseGrain
                ) {
                    ui.label(if *distort_type == DistortType::NoiseGrain { "Grain:" } else { "Radius:" });
                    drag_val(ui, DragValue::new(radius).range(0.0..=2.0).speed(0.02));
                    ui.end_row();
                }
                if matches!(
                    distort_type,
                    DistortType::FishEye
                        | DistortType::LensDistortion
                        | DistortType::Mirror
                        | DistortType::Offset
                        | DistortType::Twirl
                        | DistortType::TurbulentDisplace
                ) {
                    ui.label("Center X:");
                    drag_val(ui, DragValue::new(&mut center[0]).range(-2.0..=2.0).speed(0.01));
                    ui.end_row();
                    ui.label("Center Y:");
                    drag_val(ui, DragValue::new(&mut center[1]).range(-2.0..=2.0).speed(0.01));
                    ui.end_row();
                }
                if matches!(
                    distort_type,
                    DistortType::Vignette
                        | DistortType::Offset
                        | DistortType::FindEdges
                        | DistortType::NoiseGrain
                ) {
                    let f_label = match distort_type {
                        DistortType::Offset | DistortType::FindEdges => "Blend:",
                        DistortType::NoiseGrain => "Color:",
                        _ => "Feather:",
                    };
                    ui.label(f_label);
                    drag_val(ui, DragValue::new(feather).range(0.0..=1.0).speed(0.02));
                    ui.end_row();
                }
            });
            true
        }
        DarkNode::Transition {
            transition_type,
            progress,
            feather,
            direction,
        } => {
            ComboBox::from_id_salt(("tr_type", node_ptr))
                .selected_text(format!("{transition_type:?}"))
                .show_ui(ui, |ui| {
                    ui.selectable_value(transition_type, TransitionType::CrossDissolve, "Cross Dissolve");
                    ui.selectable_value(transition_type, TransitionType::LinearWipe, "Linear Wipe");
                    ui.selectable_value(transition_type, TransitionType::Slide, "Slide");
                    ui.selectable_value(transition_type, TransitionType::Iris, "Iris");
                    ui.selectable_value(transition_type, TransitionType::ZoomPush, "Zoom Push");
                    ui.selectable_value(transition_type, TransitionType::DipBlack, "Dip to Black");
                    ui.selectable_value(transition_type, TransitionType::DipWhite, "Dip to White");
                    ui.selectable_value(transition_type, TransitionType::BlurDissolve, "Blur Dissolve");
                });
            Grid::new(("tr_grid", node_ptr)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                ui.label("Progress:");
                drag_val(ui, DragValue::new(progress).range(0.0..=1.0).speed(0.01));
                ui.end_row();
                ui.label("Feather:");
                drag_val(ui, DragValue::new(feather).range(0.0..=1.0).speed(0.01));
                ui.end_row();
                if matches!(transition_type, TransitionType::LinearWipe | TransitionType::Slide) {
                    ui.label("Dir:");
                    ComboBox::from_id_salt(("tr_dir", node_ptr))
                        .selected_text(format!("{direction:?}"))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(direction, TransitionDirection::Left, "Left");
                            ui.selectable_value(direction, TransitionDirection::Right, "Right");
                            ui.selectable_value(direction, TransitionDirection::Up, "Up");
                            ui.selectable_value(direction, TransitionDirection::Down, "Down");
                        });
                    ui.end_row();
                }
            });
            true
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
            true
        }
        _ => false,
    }
}
