#[derive(Debug, Clone)]
pub struct DisplaySpec {
    pub index: usize,
    pub x: f32,
    pub width: f32,
}

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub fn displays() -> Vec<DisplaySpec> {
    macos::displays()
}

#[cfg(not(target_os = "macos"))]
pub fn displays() -> Vec<DisplaySpec> {
    vec![DisplaySpec {
        index: 0,
        x: 0.0,
        width: 1024.0,
    }]
}

#[cfg(target_os = "macos")]
pub fn configure_bar_window(
    window: &dyn iced::window::Window,
    bar_height: f32,
) -> Result<(), String> {
    macos::configure_bar_window(window, bar_height)
}

#[cfg(not(target_os = "macos"))]
pub fn configure_bar_window(
    _window: &dyn iced::window::Window,
    _bar_height: f32,
) -> Result<(), String> {
    Ok(())
}
// Reserved for future platform-specific UI integrations.
