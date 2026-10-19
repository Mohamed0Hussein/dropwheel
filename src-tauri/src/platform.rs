//! Win32 plumbing that Tauri does not expose.

#[cfg(windows)]
pub use windows_impl::hidden_owner;

#[cfg(windows)]
mod windows_impl {
    use std::sync::OnceLock;

    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{CreateWindowExW, WINDOW_EX_STYLE, WS_POPUP};
    use windows::core::w;

    /// A window that is never shown, used as the owner of the shelf and other floating
    /// windows.
    ///
    /// Windows leaves owned windows out of the taskbar and Alt+Tab. Tauri's `skip_taskbar`
    /// alone is not enough: an unowned window gets `WS_EX_APPWINDOW`, which keeps it in
    /// Alt+Tab, and its taskbar button can come back when it is shown.
    ///
    /// Call from the main thread so the owner is served by the app's message loop.
    pub fn hidden_owner() -> Option<HWND> {
        // HWND is a raw pointer and not Sync, so keep the handle as an integer.
        static OWNER: OnceLock<Option<usize>> = OnceLock::new();
        let raw = OWNER.get_or_init(|| {
            // SAFETY: plain window creation with a system class and no extra parameters.
            let created = unsafe {
                CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("STATIC"),
                    w!("Dropwheel owner"),
                    WS_POPUP,
                    0,
                    0,
                    0,
                    0,
                    None,
                    None,
                    None,
                    None,
                )
            };
            created
                .map(|hwnd| hwnd.0 as usize)
                .map_err(|err| log::error!("could not create the owner window: {err}"))
                .ok()
        });
        raw.map(|raw| HWND(raw as *mut _))
    }
}
