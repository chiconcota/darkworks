//! # Darkworks User Interface Crate
//!
//! Giao diện người dùng 4 phân vùng chuẩn NLE vẽ bằng `egui`:
//! - [`panels::menu_bar`]: Thanh menu lệnh và chỉ số hệ thống (RAM, FPS, GPU).
//! - [`panels::source_bin`]: Phân vùng quản lý tài nguyên video/âm thanh (Trái).
//! - [`panels::viewport`]: Màn hình xem trước video 16:9 Zero-Copy (Giữa).
//! - [`panels::inspector`]: Bảng điều khiển thuộc tính & xem trước Node Graph (Phải).
//! - [`panels::timeline`]: Trục thời gian đa track cắt ghép (Dưới).

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod app;
pub mod panels;
pub mod theme;

pub use app::DarkworksApp;
