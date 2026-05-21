pub(crate) mod cgwindow;
pub(crate) mod error;
pub(crate) mod ipc;
pub(crate) mod native;
pub(crate) mod probe;
pub(crate) mod screencapture;
pub(crate) mod window_id;

pub(crate) use cgwindow::capture_cgwindow;
pub(crate) use error::ScreenshotError;
pub(crate) use ipc::handle_screenshot;
pub(crate) use probe::{ScreenshotBackend, selected_backend};
pub(crate) use screencapture::capture_screencapture;
pub(crate) use window_id::{
    DiscoveredWindow, WindowBounds, get_window_bounds, list_layer_zero_windows,
};
