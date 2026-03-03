#[derive(Debug, Clone)]
pub struct DisplaySpec {
    pub x: f32,
    pub width: f32,
}

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
pub use macos::configure_bar_window;

#[cfg(target_os = "macos")]
pub fn primary_display() -> DisplaySpec {
    macos::primary_display()
}

#[cfg(target_os = "linux")]
pub fn primary_display() -> DisplaySpec {
    linux::primary_display_wayland()
}

#[cfg(target_os = "linux")]
pub fn configure_bar_window(window: &winit::window::Window, bar_height: f32) -> Result<(), String> {
    linux::configure_bar_window_wayland(window, bar_height)
}

#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub fn is_dark_mode() -> bool {
    macos::is_dark_mode()
}

#[cfg(target_os = "linux")]
#[allow(dead_code)]
pub fn is_dark_mode() -> bool {
    false
}

#[cfg(target_os = "macos")]
pub fn hide_from_dock() {
    macos::hide_from_dock();
}

#[cfg(target_os = "linux")]
pub fn hide_from_dock() {
    linux::hide_from_taskbar_wayland();
}
