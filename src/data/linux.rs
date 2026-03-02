//! Linux WM integration stub.
//!
//! Future: Use EWMH/ICCCM (X11) or Wayland compositor protocols (e.g. wlr-foreign-toplevel,
//! workspace info from Sway/Hyprland/i3) for workspaces, focused window, and app list.

use super::WmData;

pub async fn load_now_playing() -> Option<super::NowPlayingData> {
    None
}

pub async fn load_wm_data() -> WmData {
    WmData {
        mode: wm_mode().unwrap_or_else(|| String::from("main")),
        used_workspaces: wm_used_workspaces(),
        focused_workspace: wm_focused_workspace(),
        apps_in_focused_workspace: Vec::new(),
    }
}

pub async fn load_focused_workspace_bridge() -> Option<String> {
    // TODO(linux-wm): EWMH _NET_CURRENT_DESKTOP or Wayland compositor workspace protocol
    None
}

pub async fn load_mode_bridge() -> Option<String> {
    // TODO(linux-wm): Compositor-specific mode (e.g. Sway mode, Hyprland mode)
    None
}

pub async fn load_apps_for_workspace(_workspace: &str) -> Vec<String> {
    // TODO(linux-wm): EWMH _NET_CLIENT_LIST or Wayland wlr-foreign-toplevel
    Vec::new()
}

fn wm_mode() -> Option<String> {
    // TODO(linux-wm): Compositor mode (e.g. /tmp/sway-mode or similar)
    None
}

fn wm_used_workspaces() -> Vec<String> {
    // TODO(linux-wm): EWMH _NET_DESKTOP_VIEWPORT / _NET_NUMBER_OF_DESKTOPS or Wayland protocol
    Vec::new()
}

fn wm_focused_workspace() -> Option<String> {
    // TODO(linux-wm): EWMH _NET_CURRENT_DESKTOP or Wayland compositor workspace
    None
}
