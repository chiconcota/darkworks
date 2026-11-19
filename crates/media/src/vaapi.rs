//! Phát hiện và thăm dò khả năng giải mã phần cứng Linux VA-API (Intel / AMD Radeon / Mesa)
//! Sử dụng dynamic loading `libloading` để nạp `libva.so.2` và `libva-drm.so.2` trong thời gian chạy.

#![allow(unsafe_code)]

use std::ffi::{CStr, c_char, c_int, c_void};
use std::fs::File;
use std::os::fd::AsRawFd;
use std::path::Path;
use libloading::Library;
use crate::error::{MediaError, MediaResult};

pub type VADisplay = *mut c_void;
pub type VAStatus = c_int;

const VA_STATUS_SUCCESS: VAStatus = 0;

#[derive(Debug, Clone)]
pub struct VaApiStatus {
    pub available: bool,
    pub vendor_string: Option<String>,
    pub device_path: Option<String>,
    pub supported_codecs: Vec<String>,
}

/// Thăm dò thiết bị phần cứng VA-API trên Linux
pub fn probe_vaapi() -> VaApiStatus {
    // 1. Kiểm tra các DRM render node phổ biến
    let render_nodes = ["/dev/dri/renderD128", "/dev/dri/renderD129", "/dev/dri/card0", "/dev/dri/card1"];
    let mut selected_node: Option<String> = None;

    for node in render_nodes {
        if Path::new(node).exists() {
            selected_node = Some(node.to_string());
            break;
        }
    }

    let Some(ref dev_path) = selected_node else {
        return VaApiStatus {
            available: false,
            vendor_string: None,
            device_path: None,
            supported_codecs: Vec::new(),
        };
    };

    // 2. Thử nạp runtime libva.so.2 và libva-drm.so.2
    match load_vaapi_runtime(dev_path) {
        Ok(vendor) => {
            log::info!("VA-API Hardware Acceleration phát hiện thành công: {vendor} trên {dev_path}");
            VaApiStatus {
                available: true,
                vendor_string: Some(vendor),
                device_path: Some(dev_path.clone()),
                supported_codecs: vec![
                    "H.264 (AVC) Main/High".into(),
                    "HEVC (H.265) Main/Main10".into(),
                    "VP9 Profile 0/2".into(),
                    "AV1 Profile 0".into(),
                ],
            }
        }
        Err(err) => {
            log::warn!("VA-API không khả dụng: {err}");
            VaApiStatus {
                available: false,
                vendor_string: None,
                device_path: Some(dev_path.clone()),
                supported_codecs: Vec::new(),
            }
        }
    }
}

fn load_vaapi_runtime(device_path: &str) -> MediaResult<String> {
    // SAFETY: Nạp thư viện hệ thống libva.so.2 được cung cấp bởi OS native
    let lib_va = unsafe { Library::new("libva.so.2") }
        .map_err(|e| MediaError::HwAccel(format!("Không thể nạp libva.so.2: {e}")))?;

    // SAFETY: Nạp libva-drm.so.2 để liên kết với Linux DRM render node
    let lib_drm = unsafe { Library::new("libva-drm.so.2") }
        .map_err(|e| MediaError::HwAccel(format!("Không thể nạp libva-drm.so.2: {e}")))?;

    let file = File::open(device_path)
        .map_err(|e| MediaError::HwAccel(format!("Không thể mở {device_path}: {e}")))?;
    let fd = file.as_raw_fd();

    // SAFETY: Lấy con trỏ hàm vaGetDisplayDRM từ libva-drm.so.2
    let va_get_display_drm: libloading::Symbol<unsafe extern "C" fn(c_int) -> VADisplay> = unsafe {
        lib_drm.get(b"vaGetDisplayDRM\0")
            .map_err(|e| MediaError::HwAccel(format!("vaGetDisplayDRM: {e}")))?
    };

    // SAFETY: Gọi vaGetDisplayDRM với file descriptor hợp lệ của render node
    let dpy = unsafe { va_get_display_drm(fd) };
    if dpy.is_null() {
        return Err(MediaError::HwAccel("vaGetDisplayDRM trả về NULL".into()));
    }

    // SAFETY: Lấy hàm vaInitialize và vaQueryVendorString từ libva.so.2
    let va_initialize: libloading::Symbol<unsafe extern "C" fn(VADisplay, *mut c_int, *mut c_int) -> VAStatus> = unsafe {
        lib_va.get(b"vaInitialize\0")
            .map_err(|e| MediaError::HwAccel(format!("vaInitialize: {e}")))?
    };
    let va_query_vendor_string: libloading::Symbol<unsafe extern "C" fn(VADisplay) -> *const c_char> = unsafe {
        lib_va.get(b"vaQueryVendorString\0")
            .map_err(|e| MediaError::HwAccel(format!("vaQueryVendorString: {e}")))?
    };
    let va_terminate: libloading::Symbol<unsafe extern "C" fn(VADisplay) -> VAStatus> = unsafe {
        lib_va.get(b"vaTerminate\0")
            .map_err(|e| MediaError::HwAccel(format!("vaTerminate: {e}")))?
    };

    let mut major = 0;
    let mut minor = 0;
    // SAFETY: Khởi tạo display context VA-API
    let status = unsafe { va_initialize(dpy, &mut major, &mut minor) };
    if status != VA_STATUS_SUCCESS {
        return Err(MediaError::HwAccel(format!("vaInitialize thất bại với mã lỗi {status}")));
    }

    // SAFETY: Truy vấn thông tin vendor driver
    let vendor_ptr = unsafe { va_query_vendor_string(dpy) };
    let vendor = if !vendor_ptr.is_null() {
        // SAFETY: Chuỗi C string kết thúc bằng null trả về từ VA-API driver
        unsafe { CStr::from_ptr(vendor_ptr) }
            .to_string_lossy()
            .to_string()
    } else {
        format!("VA-API {major}.{minor} Generic Driver")
    };

    // SAFETY: Đóng display context sau khi kiểm tra xong
    unsafe {
        let _ = va_terminate(dpy);
    }

    Ok(vendor)
}
