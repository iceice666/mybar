//! Linux Wayland platform stub.
//!
//! Future integration points:
//! - **Display enumeration**: Use `wl_output` to enumerate outputs and their geometry
//! - **Bar window**: Use wlr-layer-shell (or equivalent) for top bar positioning,
//!   always-on-top, and spanning the full output width
//! - **Hide from taskbar**: Layer-shell top overlay typically does not appear in taskbars

use super::DisplaySpec;

/// Wayland: Enumerate displays via wl_output.
///
/// TODO(wayland): Connect to wl_display, bind wl_output, read output geometry
/// and transform to DisplaySpec (x, width per output).
pub fn displays_wayland() -> Vec<DisplaySpec> {
    vec![DisplaySpec {
        index: 0,
        x: 0.0,
        width: 1024.0,
    }]
}

/// Wayland: Configure bar window for layer-shell / top overlay.
///
/// TODO(wayland): Use wlr-layer-shell (zwlr_layer_surface_v1) to position the bar
/// at the top of the output, set layer to overlay or top, anchor to top edge.
pub fn configure_bar_window_wayland(
    _window: &winit::window::Window,
    _bar_height: f32,
) -> Result<(), String> {
    Ok(())
}

/// Wayland: Hide bar from taskbar / app list.
///
/// TODO(wayland): Layer-shell overlay surfaces typically do not appear in taskbars.
/// If needed, use xdg_toplevel or compositor-specific protocols.
#[allow(dead_code)]
pub fn hide_from_taskbar_wayland() {
    // No-op for now; layer-shell may handle this implicitly.
}
