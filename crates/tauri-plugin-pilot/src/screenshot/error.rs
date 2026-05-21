use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum ScreenshotError {
    // Constructed only in `cfg(not(target_os = "macos"))` stubs; the macOS
    // build still matches it so the cross-platform handler can map it to the
    // `PERMISSION_DENIED` IPC error without a per-arch dispatch table.
    #[allow(
        dead_code,
        reason = "constructed in cfg(not(target_os = \"macos\")) stubs"
    )]
    #[error("native screenshot capture is unsupported on this platform")]
    PlatformUnsupported,
    #[error("native screenshot capture failed: {message}")]
    CaptureFailed { message: String },
}
