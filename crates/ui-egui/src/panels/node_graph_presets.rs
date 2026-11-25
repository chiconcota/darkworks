//! Quản lý Lưu, Xuất/Nhập file `.preset.json` và Thư viện User Presets dùng chung giữa các Project.

use std::path::{Path, PathBuf};

use darkworks_node_graph::{
    BuiltinPresetKind, NodeGraphDoc, NodeGraphPreset, NodeGraphPresetDoc, chain_node_graphs,
    doc_from_snarl, ensure_preset_extension, sanitize_preset_filename, snarl_from_doc,
};
use egui::{Color32, RichText, Ui, vec2};

use super::node_graph::{self, NodeGraphState};
use crate::FilmcraftApp;
use crate::native_dialogs::FileDialog;

/// Chế độ nạp Preset vào đồ thị hiện tại.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PresetApplyMode {
    /// Thay thế toàn bộ đồ thị hiện tại (tách về chế độ Nháp để không ghi đè clip cũ).
    #[default]
    Replace,
    /// Nối tiếp hiệu ứng của Preset vào trước `Final Output` của đồ thị đang mở.
    Chain,
}

/// Một User Preset được quét từ thư mục dùng chung `<data_dir>/Node Presets`.
#[derive(Debug, Clone)]
pub struct UserPresetEntry {
    pub path: PathBuf,
    pub doc: NodeGraphPresetDoc,
}

/// Trạng thái UI của tính năng Lưu / Nạp Preset.
#[derive(Debug, Clone)]
pub struct PresetUiState {
    pub apply_mode: PresetApplyMode,
    pub show_save_dialog: bool,
    pub save_name: String,
    pub save_category: String,
    pub save_description: String,
    pub user_presets: Vec<UserPresetEntry>,
    pub presets_loaded: bool,
}

impl Default for PresetUiState {
    fn default() -> Self {
        Self {
            apply_mode: PresetApplyMode::Replace,
            show_save_dialog: false,
            save_name: String::new(),
            save_category: "Custom".to_string(),
            save_description: String::new(),
            user_presets: Vec::new(),
            presets_loaded: false,
        }
    }
}

/// Trả về thư mục lưu User Node Presets dùng chung toàn hệ thống (không phụ thuộc project).
pub fn user_presets_dir(app: &FilmcraftApp) -> Option<PathBuf> {
    app.session
        .presets
        .data_dir()
        .map(|d| d.join("Node Presets"))
        .or_else(|| filmcraft_engine::autosave::default_data_dir().map(|d| d.join("Node Presets")))
}

/// Quét lại toàn bộ file `.preset.json` / `.json` trong thư mục User Presets.
pub fn refresh_user_presets(app: &FilmcraftApp, ui_state: &mut PresetUiState) {
    ui_state.user_presets.clear();
    ui_state.presets_loaded = true;
    let Some(dir) = user_presets_dir(app) else {
        return;
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let is_json = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"));
        if !is_json {
            continue;
        }
        if let Ok(doc) = NodeGraphPresetDoc::load_from_file(&path) {
            ui_state.user_presets.push(UserPresetEntry { path, doc });
        }
    }
    ui_state
        .user_presets
        .sort_by(|a, b| a.doc.category.cmp(&b.doc.category).then_with(|| a.doc.name.cmp(&b.doc.name)));
}

fn ensure_presets_loaded(app: &FilmcraftApp, state: &mut NodeGraphState) {
    if !state.preset_ui.presets_loaded {
        refresh_user_presets(app, &mut state.preset_ui);
    }
}

/// Áp dụng một [`NodeGraphDoc`] vào `NodeGraphState` theo chế độ `Replace` hoặc `Chain`.
pub fn apply_graph_doc(
    state: &mut NodeGraphState,
    incoming_doc: &NodeGraphDoc,
    label: &str,
    mode: PresetApplyMode,
) {
    state.push_undo();
    let final_doc = match mode {
        PresetApplyMode::Replace => {
            state.detach_to_scratchpad();
            incoming_doc.clone()
        }
        PresetApplyMode::Chain => {
            let current = doc_from_snarl(&state.snarl, &state.groups, &state.custom_titles);
            chain_node_graphs(&current, incoming_doc)
        }
    };
    let (snarl, groups, custom_titles) = snarl_from_doc(&final_doc);
    state.snarl = snarl;
    state.groups = groups;
    state.custom_titles = custom_titles;
    state.selected_group = None;
    state.last_hovered_group = None;
    state.status_message = match mode {
        PresetApplyMode::Replace => format!(
            "Đã nạp Preset '{label}' (Nháp — bấm 'Add as FX Clip' hoặc 'Apply to Clip')"
        ),
        PresetApplyMode::Chain => format!("✔ Đã nối tiếp Preset '{label}' trước Final Output"),
    };
}

/// Mở hộp thoại Lưu Preset vào Thư viện dùng chung hoặc Xuất file `.preset.json`.
pub fn open_save_preset_dialog(state: &mut NodeGraphState) {
    if state.preset_ui.save_name.trim().is_empty() {
        let suggested = state
            .groups
            .first()
            .map(|g| g.title.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                if !state.bound_clip_name.trim().is_empty() {
                    Some(format!("{} Preset", state.bound_clip_name.trim()))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "My Node Preset".to_string());
        state.preset_ui.save_name = suggested;
    }
    if state.preset_ui.save_category.trim().is_empty() {
        state.preset_ui.save_category = "Custom".to_string();
    }
    state.preset_ui.show_save_dialog = true;
}

/// Mở hộp thoại `Save As` để xuất file `.preset.json` ra vị trí bất kỳ trên ổ đĩa.
pub fn trigger_export_preset_dialog(
    app: &mut FilmcraftApp,
    ctx: &egui::Context,
    state: &mut NodeGraphState,
    name: &str,
    category: &str,
    description: &str,
) {
    let graph = doc_from_snarl(&state.snarl, &state.groups, &state.custom_titles);
    if graph.is_empty() {
        state.status_message = "✖ Đồ thị đang rỗng, chưa thể xuất Preset".to_string();
        return;
    }
    let clean_name = if name.trim().is_empty() {
        "Darkworks Preset"
    } else {
        name.trim()
    };
    let preset = NodeGraphPresetDoc::new(clean_name, category.trim(), description.trim(), graph);
    let suggested = sanitize_preset_filename(clean_name);
    let ctx_clone = ctx.clone();

    app.pick_ui(
        FileDialog::save_as_named("Darkworks Node Preset", &["json"], &suggested),
        move |_app, paths| {
            let Some(raw_path) = paths.into_iter().next() else {
                return;
            };
            let final_path = ensure_preset_extension(&raw_path);
            let msg = match preset.save_to_file(Path::new(&final_path)) {
                Ok(()) => format!("✔ Đã xuất Preset ra file: {final_path}"),
                Err(e) => format!("✖ Lỗi xuất Preset: {e}"),
            };
            node_graph::with_state(&ctx_clone, |st| {
                st.status_message = msg;
            });
            ctx_clone.request_repaint();
        },
    );
}

/// Mở hộp thoại `Open File` để nạp file `.preset.json` từ ổ đĩa vào chế độ Nháp.
pub fn trigger_import_preset_dialog(app: &mut FilmcraftApp, ctx: &egui::Context) {
    let ctx_clone = ctx.clone();
    app.pick_ui(
        FileDialog::open_file("Darkworks Node Preset", &["json"]),
        move |_app, paths| {
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            match NodeGraphPresetDoc::load_from_file(Path::new(&path)) {
                Ok(preset) => {
                    node_graph::with_state(&ctx_clone, |st| {
                        apply_graph_doc(st, &preset.graph, &preset.name, PresetApplyMode::Replace);
                        st.status_message = format!(
                            "✔ Đã Import Preset '{}' từ file (Chế độ Nháp)",
                            preset.name
                        );
                    });
                }
                Err(e) => {
                    node_graph::with_state(&ctx_clone, |st| {
                        st.status_message = format!("✖ Không thể đọc file Preset: {e}");
                    });
                }
            }
            ctx_clone.request_repaint();
        },
    );
}

/// Hiển thị Menu `📂 Load Preset` gồm chế độ Replace/Chain, 5 Built-in Presets và Thư viện User Presets.
pub fn render_load_preset_menu(app: &mut FilmcraftApp, ui: &mut Ui, state: &mut NodeGraphState) {
    ensure_presets_loaded(app, state);
    let mut reveal_dir: Option<PathBuf> = None;
    let mut delete_path: Option<PathBuf> = None;
    let mut should_import = false;
    let mut should_save = false;

    ui.menu_button("📂 Load Preset", |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Chế độ áp:").small().weak());
            ui.selectable_value(&mut state.preset_ui.apply_mode, PresetApplyMode::Replace, "🔄 Thay thế")
                .on_hover_text("Thay toàn bộ đồ thị bằng Preset (tách về chế độ Nháp)");
            ui.selectable_value(&mut state.preset_ui.apply_mode, PresetApplyMode::Chain, "➕ Nối tiếp (Chain)")
                .on_hover_text("Nối tiếp Preset vào trước Final Output của đồ thị đang mở");
        });
        ui.separator();

        ui.label(RichText::new("--- PRESET MẪU (BUILT-IN) ---").weak().small());
        const PRESETS: &[(&str, BuiltinPresetKind, &str)] = &[
            ("1. 📰 News Ticker Banner", BuiltinPresetKind::NewsTickerBanner, "News Ticker Banner"),
            ("2. 🌊 Audio Waveform Line (60 FPS)", BuiltinPresetKind::ContinuousAudioWaveform, "Continuous Audio Waveform Line"),
            ("3. 🎯 Spotlight Invert", BuiltinPresetKind::SpotlightInvert, "Spotlight Invert"),
            ("4. ⚡ Audio Beat Invert Flash", BuiltinPresetKind::AudioBeatFlash, "Audio Beat Invert Flash"),
            ("5. ✨ Cinematic Soft Glow", BuiltinPresetKind::CinematicSoftGlow, "Cinematic Soft Glow"),
        ];
        let mode = state.preset_ui.apply_mode;
        for &(btn_text, kind, label) in PRESETS {
            if ui.button(btn_text).clicked() {
                let doc = NodeGraphPreset::builtin_doc(kind);
                apply_graph_doc(state, &doc, label, mode);
                ui.close();
            }
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.label(RichText::new("--- THƯ VIỆN CỦA BẠN (DÙNG MỌI PROJECT) ---").weak().small());
            if ui.small_button("🔄").on_hover_text("Quét lại thư mục User Presets").clicked() {
                refresh_user_presets(app, &mut state.preset_ui);
            }
        });

        if state.preset_ui.user_presets.is_empty() {
            ui.label(RichText::new("  (Chưa có Preset lưu sẵn — bấm '💾 Save Preset' để lưu)").weak().small());
        } else {
            let entries = state.preset_ui.user_presets.clone();
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                for entry in &entries {
                    ui.horizontal(|ui| {
                        let label = format!("[{}] {}", entry.doc.category, entry.doc.name);
                        let tip = if entry.doc.description.is_empty() {
                            format!("File: {}", entry.path.display())
                        } else {
                            format!("{}\nFile: {}", entry.doc.description, entry.path.display())
                        };
                        if ui.button(label).on_hover_text(tip).clicked() {
                            apply_graph_doc(state, &entry.doc.graph, &entry.doc.name, mode);
                            ui.close();
                        }
                        if ui
                            .small_button(RichText::new("🗑").color(Color32::from_rgb(255, 110, 110)))
                            .on_hover_text("Xóa Preset này khỏi thư viện")
                            .clicked()
                        {
                            delete_path = Some(entry.path.clone());
                        }
                    });
                }
            });
        }

        ui.separator();
        if ui.button("💾 Lưu vào Thư viện / Xuất file…").clicked() {
            should_save = true;
            ui.close();
        }
        if ui.button("📥 Import từ file .preset.json…").clicked() {
            should_import = true;
            ui.close();
        }
        if ui.button("📂 Mở thư mục User Presets").clicked()
            && let Some(dir) = user_presets_dir(app)
        {
            let _ = std::fs::create_dir_all(&dir);
            reveal_dir = Some(dir);
            ui.close();
        }
    });

    if let Some(path) = delete_path {
        match std::fs::remove_file(&path) {
            Ok(()) => {
                state.status_message = format!("🗑 Đã xóa Preset: {}", path.display());
                refresh_user_presets(app, &mut state.preset_ui);
            }
            Err(e) => {
                state.status_message = format!("✖ Không thể xóa Preset: {e}");
            }
        }
    }
    if should_save {
        open_save_preset_dialog(state);
    }
    if should_import {
        trigger_import_preset_dialog(app, ui.ctx());
    }
    if let Some(dir) = reveal_dir
        && let Err(e) = crate::panels::menu_dialogs::open_path(app, ui.ctx(), &dir.to_string_lossy(), false)
    {
        state.status_message = format!("✖ Không mở được thư mục Presets: {e}");
    }
}

/// Hiển thị cửa sổ modal Lưu Preset vào Thư viện (dùng chung các Project) hoặc Export ra file rời.
pub fn show_save_preset_dialog(
    app: &mut FilmcraftApp,
    state: &mut NodeGraphState,
    ctx: &egui::Context,
) {
    if !state.preset_ui.show_save_dialog {
        return;
    }
    let mut open = state.preset_ui.show_save_dialog;
    let mut save_to_library = false;
    let mut export_to_file = false;
    let mut copy_clipboard = false;

    egui::Window::new("💾 Lưu Node Graph Preset")
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .default_size(vec2(420.0, 260.0))
        .show(ctx, |ui| {
            ui.label(
                RichText::new("Preset được lưu dạng .preset.json độc lập — dùng lại ở mọi Project khác.")
                    .small()
                    .weak(),
            );
            ui.add_space(6.0);

            egui::Grid::new("save_preset_grid")
                .num_columns(2)
                .spacing([8.0, 6.0])
                .show(ui, |ui| {
                    ui.label("Tên Preset:");
                    ui.text_edit_singleline(&mut state.preset_ui.save_name);
                    ui.end_row();

                    ui.label("Phân loại:");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut state.preset_ui.save_category);
                    });
                    ui.end_row();
                });

            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Gợi ý:").small().weak());
                for cat in ["Custom", "Color", "Blur & Glow", "Overlay / Text", "Audio FX"] {
                    if ui.small_button(cat).clicked() {
                        state.preset_ui.save_category = cat.to_string();
                    }
                }
            });

            ui.add_space(4.0);
            ui.label("Mô tả:");
            ui.add(
                egui::TextEdit::multiline(&mut state.preset_ui.save_description)
                    .desired_rows(2)
                    .desired_width(f32::INFINITY),
            );

            let preview_file = sanitize_preset_filename(&state.preset_ui.save_name);
            if let Some(dir) = user_presets_dir(app) {
                ui.label(
                    RichText::new(format!("Thư mục chung: {}/{}", dir.display(), preview_file))
                        .small()
                        .weak(),
                );
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button(RichText::new("📚 Lưu vào Thư viện").strong()).clicked() {
                    save_to_library = true;
                }
                if ui.button("💾 Xuất file .preset.json…").clicked() {
                    export_to_file = true;
                }
                if ui.button("📋 Copy JSON").clicked() {
                    copy_clipboard = true;
                }
            });
        });

    if save_to_library {
        let graph = doc_from_snarl(&state.snarl, &state.groups, &state.custom_titles);
        if graph.is_empty() {
            state.status_message = "✖ Đồ thị đang rỗng, chưa thể lưu Preset".to_string();
        } else if let Some(dir) = user_presets_dir(app) {
            let name = if state.preset_ui.save_name.trim().is_empty() {
                "Custom Preset".to_string()
            } else {
                state.preset_ui.save_name.trim().to_string()
            };
            let cat = if state.preset_ui.save_category.trim().is_empty() {
                "Custom".to_string()
            } else {
                state.preset_ui.save_category.trim().to_string()
            };
            let desc = state.preset_ui.save_description.trim().to_string();
            let preset = NodeGraphPresetDoc::new(name.clone(), cat, desc, graph);
            let file_path = dir.join(sanitize_preset_filename(&name));
            match preset.save_to_file(&file_path) {
                Ok(()) => {
                    refresh_user_presets(app, &mut state.preset_ui);
                    state.status_message =
                        format!("✔ Đã lưu Preset '{name}' vào Thư viện dùng chung ({})", file_path.display());
                    open = false;
                }
                Err(e) => {
                    state.status_message = format!("✖ Lỗi lưu Preset: {e}");
                }
            }
        } else {
            state.status_message = "✖ Không tìm thấy thư mục dữ liệu ứng dụng để lưu Preset".to_string();
        }
    }

    if export_to_file {
        let name = state.preset_ui.save_name.clone();
        let cat = state.preset_ui.save_category.clone();
        let desc = state.preset_ui.save_description.clone();
        trigger_export_preset_dialog(app, ctx, state, &name, &cat, &desc);
        open = false;
    }

    if copy_clipboard {
        let graph = doc_from_snarl(&state.snarl, &state.groups, &state.custom_titles);
        let name = if state.preset_ui.save_name.trim().is_empty() {
            "Darkworks Preset"
        } else {
            state.preset_ui.save_name.trim()
        };
        let preset = NodeGraphPresetDoc::new(
            name,
            state.preset_ui.save_category.trim(),
            state.preset_ui.save_description.trim(),
            graph,
        );
        if let Ok(json) = preset.to_json_string() {
            ctx.copy_text(json);
            state.status_message = "✔ Đã copy Preset JSON đầy đủ (kèm dây nối) vào Clipboard".to_string();
        }
    }

    state.preset_ui.show_save_dialog = open;
}
