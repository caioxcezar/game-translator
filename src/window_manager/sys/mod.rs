#[cfg(target_os = "linux")]
mod linux;

#[cfg(windows)]
mod windows;

mod platform {
    #[cfg(target_os = "linux")]
    pub use super::linux::window_manager as WindowManager;

    #[cfg(windows)]
    pub use super::windows::window_manager as WindowManager;
}

pub use platform::WindowManager;
