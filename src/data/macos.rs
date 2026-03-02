//! macOS-specific data sources: AeroSpace WM, nowplaying-cli, Core WLAN.

use objc2::rc::autoreleasepool;
use objc2_core_wlan::CWWiFiClient;

use super::{NowPlayingData, WmData};

pub async fn load_now_playing() -> Option<NowPlayingData> {
    let output = tokio::process::Command::new("nowplaying-cli")
        .args(["get", "title", "artist"])
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let content = String::from_utf8(output.stdout).ok()?;
    let mut lines = content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(ToOwned::to_owned);
    let title = lines.next().unwrap_or_default();
    let artist = lines.next().unwrap_or_default();
    if title.is_empty() && artist.is_empty() {
        None
    } else {
        Some(NowPlayingData { title, artist })
    }
}

pub async fn load_wm_data() -> WmData {
    WmData {
        mode: wm_mode().unwrap_or_else(|| String::from("main")),
        used_workspaces: wm_used_workspaces(),
        focused_workspace: wm_focused_workspace(),
        apps_in_focused_workspace: Vec::new(),
    }
}

pub async fn load_apps_for_workspace(workspace: &str) -> Vec<String> {
    let preferred = run_aerospace(&[
        "list-windows",
        "--workspace",
        workspace,
        "--format",
        "%{app-name}",
    ])
    .map(parse_lines)
    .unwrap_or_default();
    if !preferred.is_empty() {
        return unique_preserve_order(preferred);
    }
    let fallback = run_aerospace(&["list-windows", "--workspace", workspace])
        .map(parse_lines)
        .unwrap_or_default();
    let parsed: Vec<String> = fallback
        .iter()
        .filter_map(|line| line.split('|').nth(1))
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
        .collect();
    unique_preserve_order(parsed)
}

fn wm_mode() -> Option<String> {
    std::fs::read_to_string("/tmp/aerospace-mode")
        .ok()
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

fn wm_used_workspaces() -> Vec<String> {
    let preferred = run_aerospace(&["list-windows", "--all", "--format", "%{workspace}"])
        .map(parse_lines)
        .unwrap_or_default();
    if !preferred.is_empty() {
        return super::unique_sorted_workspaces(preferred);
    }
    let fallback = run_aerospace(&["list-workspaces", "--monitor", "all"])
        .map(parse_lines)
        .unwrap_or_default();
    super::unique_sorted_workspaces(fallback)
}

fn wm_focused_workspace() -> Option<String> {
    run_aerospace(&["list-workspaces", "--focused"])
        .and_then(|o| parse_lines(o).into_iter().next())
}

fn run_aerospace(args: &[&str]) -> Option<String> {
    let output = std::process::Command::new("aerospace")
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|s| s.trim().to_owned())
}

fn parse_lines(output: String) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn unique_preserve_order(input: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    input
        .into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
}

/// Read WiFi signal strength (0–100). Returns None when not on WiFi or unavailable.
/// Uses Core WLAN (no sudo, no subprocess).
pub fn read_wifi_signal() -> Option<u8> {
    autoreleasepool(|_| {
        let client = unsafe { CWWiFiClient::sharedWiFiClient() };
        let iface = unsafe { client.interface() }?;
        let rssi = unsafe { iface.rssiValue() };
        // 0 means error or not associated (Core WLAN docs).
        if rssi == 0 {
            return None;
        }
        let rssi_i32 = rssi as i32;
        Some(rssi_to_percent(rssi_i32))
    })
}

fn rssi_to_percent(rssi_dbm: i32) -> u8 {
    // RSSI typically -90 (weak) to -30 (strong). Linear map to 0..=100.
    let clamped = rssi_dbm.clamp(-90, -30);
    let pct = (clamped + 90) as f64 / 60.0 * 100.0;
    pct.round().clamp(0.0, 100.0) as u8
}
