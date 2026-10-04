#[cfg(target_os = "windows")]
use gpui_kit::Window;

pub(crate) fn platform_notice() -> Option<String> {
    #[cfg(target_os = "macos")]
    if !macos_accessibility_client::accessibility::application_is_trusted() {
        return Some("Enable Accessibility for KeyJolt (or the terminal that launched it) in System Settings > Privacy & Security > Accessibility. Global sounds require this permission.".to_string());
    }
    #[cfg(target_os = "linux")]
    if std::env::var("XDG_SESSION_TYPE")
        .is_ok_and(|session| session.eq_ignore_ascii_case("wayland"))
    {
        return Some("Global sounds may not work in a Wayland session. Log in to an X11 session to use the global input listener.".to_string());
    }
    None
}

#[cfg(target_os = "windows")]
pub(crate) fn set_windows_window_visibility(window: &Window, visible: bool) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows_sys::Win32::UI::WindowsAndMessaging::{SW_HIDE, SW_SHOW, ShowWindow};

    let Ok(handle) = <Window as HasWindowHandle>::window_handle(window) else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };
    let command = if visible { SW_SHOW } else { SW_HIDE };
    // SAFETY: GPUI supplies a live HWND for this window, and this only changes its visibility on the UI thread.
    unsafe {
        ShowWindow(handle.hwnd.get() as _, command);
    }
}
