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

/// Read WiFi signal strength (0–100). Returns None when not on WiFi or unavailable.
/// Reads /proc/net/wireless (link quality) or tries nmcli for the in-use AP signal.
pub fn read_wifi_signal() -> Option<u8> {
    proc_net_wireless_signal().or_else(nmcli_wifi_signal)
}

fn proc_net_wireless_signal() -> Option<u8> {
    let content = std::fs::read_to_string("/proc/net/wireless").ok()?;
    // Lines: Inter-| sta-|   Quality ... ; then wlan0: 0000. 70. 70. ...
    for line in content.lines().skip(2) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let _iface = parts.next()?;
        let _status = parts.next()?;
        let quality = parts.next()?;
        let quality = quality.trim_end_matches('.').parse::<u8>().ok()?;
        // Kernel reports 0–70 typically; normalize to 0–100
        let pct = (quality as f64 / 70.0 * 100.0).round().clamp(0.0, 100.0) as u8;
        return Some(pct);
    }
    None
}

fn nmcli_wifi_signal() -> Option<u8> {
    let output = std::process::Command::new("nmcli")
        .args(["-t", "-f", "IN-USE,SIGNAL", "dev", "wifi", "list"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8(output.stdout).ok()?;
    for line in s.lines() {
        let line = line.trim();
        if line.starts_with("*:") {
            let signal = line.strip_prefix("*:")?.trim().parse::<u8>().ok()?;
            return Some(signal.min(100));
        }
    }
    None
}
