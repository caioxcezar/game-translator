pub mod window_manager {
    use gdk4_win32::{
        windows::{
            core::s,
            Win32::{
                Foundation::{ LPARAM, WPARAM },
                UI::WindowsAndMessaging::{
                    FindWindowA,
                    PostMessageA,
                    SetWindowLongPtrA,
                    SetWindowPos,
                    GWL_EXSTYLE,
                    HWND_TOPMOST,
                    SWP_NOACTIVATE,
                    SWP_NOMOVE,
                    SWP_NOSIZE,
                    WM_CLOSE,
                },
            },
        },
        HWND,
    };

    pub fn find_window(window_name: &str) -> HWND {
        let (class_name, window_name) = match window_name {
            "GT Overlay" => (s!("gdkSurfaceToplevel"), s!("GT Overlay")),
            _ => (s!("None"), s!("None")),
        };
        unsafe { FindWindowA(class_name, window_name) }
    }

    pub fn set_window_translucent(window_name: &str, intangible: bool) {
        unsafe {
            let hwnd = find_window(window_name);
            // WS_EX_TRANSPARENT = 32
            // WS_EX_LAYERED = 524288
            let dwnewlong = if intangible { 32 | 524288 } else { 32 };
            let _ = SetWindowLongPtrA(hwnd, GWL_EXSTYLE, dwnewlong);
            let _ = SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE
            );
        }
    }

    pub fn close_window(window_name: &str) {
        let hwnd = find_window(window_name);
        unsafe {
            let _ = PostMessageA(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }
}
