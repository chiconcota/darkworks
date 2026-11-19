//! # Darkworks Desktop Application Entry Point
//!
//! Khởi chạy cửa sổ NLE bằng `eframe` + `wgpu`.

#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]
#![deny(clippy::unwrap_used, clippy::expect_used)]

use darkworks_ui::DarkworksApp;
use egui::ViewportBuilder;
use std::sync::Arc;

fn load_icon() -> Option<Arc<egui::IconData>> {
    let icon_bytes = include_bytes!("../../../assets/branding/darkworks-logo-clean.png");
    match image::load_from_memory(icon_bytes) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            Some(Arc::new(egui::IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            }))
        }
        Err(e) => {
            log::warn!("Không thể nạp logo app icon: {e}");
            None
        }
    }
}

fn main() -> eframe::Result {
    // Khởi tạo logger an toàn tuyệt đối (mặc định RUST_LOG=info)
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("⚡ Khởi động Darkworks NLE Video Editor (Pure Rust + wgpu)...");

    let mut viewport = ViewportBuilder::default()
        .with_title("Darkworks — GPU Node-Driven NLE Video Editor")
        .with_inner_size([1440.0, 900.0])
        .with_min_inner_size([1024.0, 640.0])
        .with_active(true);

    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "darkworks",
        native_options,
        Box::new(|cc| Ok(Box::new(DarkworksApp::new(cc)))),
    )
}
