//! Data collection: all widget state gathered by tokio tasks, exposed via watch channels.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::watch;

// ── Shared data snapshot ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BarData {
    pub wm: WmData,
    pub now_playing: Option<NowPlayingData>,
    pub cpu_percent: f32,
    pub mem_used: u64,
    pub mem_total: u64,
    pub net_upload: f64,
    pub net_download: f64,
    pub battery_percent: Option<u8>,
    pub battery_charging: bool,
    pub date: String,
    pub time: String,
}

impl Default for BarData {
    fn default() -> Self {
        let now = chrono::Local::now();
        Self {
            wm: WmData::default(),
            now_playing: None,
            cpu_percent: 0.0,
            mem_used: 0,
            mem_total: 0,
            net_upload: 0.0,
            net_download: 0.0,
            battery_percent: None,
            battery_charging: false,
            date: now.format("%a %b %d").to_string(),
            time: now.format("%H:%M").to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct WmData {
    pub mode: String,
    pub used_workspaces: Vec<String>,
    pub focused_workspace: Option<String>,
    pub apps_in_focused_workspace: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NowPlayingData {
    pub title: String,
    pub artist: String,
}

// ── Notification channel ──────────────────────────────────────────────────────

/// A simple flag to tell the event loop a redraw is needed.
pub type RedrawNotifier = Arc<dyn Fn() + Send + Sync>;

// ── Spawn all data-collection tasks ───────────────────────────────────────────

/// Spawns tokio tasks for every data source. Returns a watch receiver that
/// always holds the latest `BarData` snapshot.
pub fn spawn_collectors(
    rt: &tokio::runtime::Handle,
    notifier: RedrawNotifier,
) -> watch::Receiver<BarData> {
    let (tx, rx) = watch::channel(BarData::default());
    let tx = Arc::new(tx);

    // Clock – 1 s
    {
        let tx = tx.clone();
        let notifier = notifier.clone();
        rt.spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                let now = chrono::Local::now();
                let date = now.format("%a %b %d").to_string();
                let time = now.format("%H:%M").to_string();
                tx.send_modify(|d| {
                    d.date = date;
                    d.time = time;
                });
                notifier();
            }
        });
    }

    // Battery – 30 s
    {
        let tx = tx.clone();
        let notifier = notifier.clone();
        rt.spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                // battery::Manager is !Send so we read on a blocking thread
                let result = tokio::task::spawn_blocking(|| {
                    let manager = battery::Manager::new().ok();
                    read_battery(&manager)
                })
                .await
                .unwrap_or((None, false));
                tx.send_modify(|d| {
                    d.battery_percent = result.0;
                    d.battery_charging = result.1;
                });
                notifier();
            }
        });
    }

    // Perf (CPU / RAM) – 2 s
    {
        let tx = tx.clone();
        let notifier = notifier.clone();
        rt.spawn(async move {
            use sysinfo::System;
            let mut sys = System::new_all();
            let mem_total = sys.total_memory();
            tx.send_modify(|d| d.mem_total = mem_total);

            let mut interval = tokio::time::interval(Duration::from_secs(2));
            loop {
                interval.tick().await;
                sys.refresh_cpu_all();
                sys.refresh_memory();
                let cpu = sys.global_cpu_usage();
                let mem = sys.used_memory();
                tx.send_modify(|d| {
                    d.cpu_percent = cpu;
                    d.mem_used = mem;
                });
                notifier();
            }
        });
    }

    // Network – 2 s with EWMA
    {
        let tx = tx.clone();
        let notifier = notifier.clone();
        rt.spawn(async move {
            use sysinfo::Networks;
            let mut networks = Networks::new_with_refreshed_list();
            let _ = net_totals(&mut networks); // prime
            let mut last = Instant::now();
            let mut smooth_up = 0.0_f64;
            let mut smooth_down = 0.0_f64;

            let mut interval = tokio::time::interval(Duration::from_secs(2));
            loop {
                interval.tick().await;
                networks.refresh(true);
                let now = Instant::now();
                let elapsed = (now - last).as_secs_f64().max(f64::EPSILON);
                last = now;

                let (rx, tx_bytes) = net_totals(&mut networks);
                smooth_down = ewma(smooth_down, rx as f64 / elapsed, elapsed);
                smooth_up = ewma(smooth_up, tx_bytes as f64 / elapsed, elapsed);

                tx.send_modify(|d| {
                    d.net_upload = smooth_up;
                    d.net_download = smooth_down;
                });
                notifier();
            }
        });
    }

    // Now Playing – 2 s
    {
        let tx = tx.clone();
        let notifier = notifier.clone();
        rt.spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(2));
            loop {
                interval.tick().await;
                let data = load_now_playing().await;
                tx.send_modify(|d| d.now_playing = data);
                notifier();
            }
        });
    }

    // WM (AeroSpace) – fast tick 500ms + fallback 10s
    {
        let tx = tx.clone();
        let notifier = notifier.clone();
        rt.spawn(async move {
            // Initial full load
            let data = load_wm_data().await;
            let focused = data.focused_workspace.clone();
            tx.send_modify(|d| d.wm = data);
            if let Some(ws) = &focused {
                let apps = load_apps_for_workspace(ws).await;
                tx.send_modify(|d| d.wm.apps_in_focused_workspace = apps);
            }
            notifier();

            let mut fast = tokio::time::interval(Duration::from_millis(500));
            let mut slow = tokio::time::interval(Duration::from_secs(10));

            loop {
                tokio::select! {
                    _ = fast.tick() => {
                        // Fast bridge reads
                        let focused = load_focused_workspace_bridge().await;
                        let mode = load_mode_bridge().await;
                        let mut changed = false;

                        let prev_focused = tx.borrow().wm.focused_workspace.clone();

                        tx.send_modify(|d| {
                            if let Some(ref m) = mode {
                                if d.wm.mode != *m {
                                    d.wm.mode = m.clone();
                                    changed = true;
                                }
                            }
                            if focused != d.wm.focused_workspace {
                                // Ensure the workspace appears in the list
                                if let Some(ref ws) = focused {
                                    if !d.wm.used_workspaces.iter().any(|w| w == ws) {
                                        d.wm.used_workspaces.push(ws.clone());
                                        d.wm.used_workspaces = unique_sorted_workspaces(
                                            std::mem::take(&mut d.wm.used_workspaces),
                                        );
                                    }
                                }
                                d.wm.focused_workspace = focused.clone();
                                changed = true;
                            }
                        });

                        if changed {
                            // Reload apps for the newly focused workspace
                            if let Some(ref ws) = focused {
                                if focused != prev_focused {
                                    let apps = load_apps_for_workspace(ws).await;
                                    tx.send_modify(|d| d.wm.apps_in_focused_workspace = apps);
                                }
                            }
                            notifier();
                        }
                    }
                    _ = slow.tick() => {
                        // Full refresh
                        let data = load_wm_data().await;
                        let focused = data.focused_workspace.clone();
                        tx.send_modify(|d| d.wm = data);
                        if let Some(ws) = &focused {
                            let apps = load_apps_for_workspace(ws).await;
                            tx.send_modify(|d| d.wm.apps_in_focused_workspace = apps);
                        }
                        notifier();
                    }
                }
            }
        });
    }

    rx
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn read_battery(manager: &Option<battery::Manager>) -> (Option<u8>, bool) {
    let Some(manager) = manager else {
        return (None, false);
    };
    let Ok(mut batteries) = manager.batteries() else {
        return (None, false);
    };
    let Some(Ok(battery)) = batteries.next() else {
        return (None, false);
    };
    let pct = (battery.state_of_charge().value * 100.0)
        .clamp(0.0, 100.0)
        .round() as u8;
    let charging = matches!(battery.state(), battery::State::Charging);
    (Some(pct), charging)
}

fn net_totals(networks: &mut sysinfo::Networks) -> (u64, u64) {
    networks.iter().fold((0u64, 0u64), |(rx, tx), (_, data)| {
        (
            rx.saturating_add(data.received()),
            tx.saturating_add(data.transmitted()),
        )
    })
}

const NET_EWMA_TAU_SECS: f64 = 4.0;
fn ewma(history: f64, sample: f64, elapsed: f64) -> f64 {
    let w = (-elapsed / NET_EWMA_TAU_SECS).exp();
    sample + (history - sample) * w
}

// ── Now Playing ───────────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
async fn load_now_playing() -> Option<NowPlayingData> {
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

#[cfg(not(target_os = "macos"))]
async fn load_now_playing() -> Option<NowPlayingData> {
    None
}

// ── WM / AeroSpace ───────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
async fn load_wm_data() -> WmData {
    WmData {
        mode: wm_mode().unwrap_or_else(|| String::from("main")),
        used_workspaces: wm_used_workspaces(),
        focused_workspace: wm_focused_workspace(),
        apps_in_focused_workspace: Vec::new(),
    }
}

#[cfg(not(target_os = "macos"))]
async fn load_wm_data() -> WmData {
    WmData::default()
}

#[cfg(target_os = "macos")]
async fn load_focused_workspace_bridge() -> Option<String> {
    let content = std::fs::read_to_string("/tmp/mybar-aerospace-focused-workspace").ok()?;
    parse_focused_workspace_bridge(content)
}

#[cfg(not(target_os = "macos"))]
async fn load_focused_workspace_bridge() -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
async fn load_mode_bridge() -> Option<String> {
    let content = std::fs::read_to_string("/tmp/aerospace-mode").ok()?;
    parse_mode_bridge(content)
}

#[cfg(not(target_os = "macos"))]
async fn load_mode_bridge() -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
async fn load_apps_for_workspace(workspace: &str) -> Vec<String> {
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

#[cfg(not(target_os = "macos"))]
async fn load_apps_for_workspace(_workspace: &str) -> Vec<String> {
    Vec::new()
}

// ── AeroSpace CLI helpers (macOS only) ────────────────────────────────────────

#[cfg(target_os = "macos")]
fn wm_mode() -> Option<String> {
    std::fs::read_to_string("/tmp/aerospace-mode")
        .ok()
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

#[cfg(target_os = "macos")]
fn wm_used_workspaces() -> Vec<String> {
    let preferred = run_aerospace(&["list-windows", "--all", "--format", "%{workspace}"])
        .map(parse_lines)
        .unwrap_or_default();
    if !preferred.is_empty() {
        return unique_sorted_workspaces(preferred);
    }
    let fallback = run_aerospace(&["list-workspaces", "--monitor", "all"])
        .map(parse_lines)
        .unwrap_or_default();
    unique_sorted_workspaces(fallback)
}

#[cfg(target_os = "macos")]
fn wm_focused_workspace() -> Option<String> {
    run_aerospace(&["list-workspaces", "--focused"])
        .and_then(|o| parse_lines(o).into_iter().next())
}

#[cfg(target_os = "macos")]
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

#[cfg(target_os = "macos")]
fn parse_lines(output: String) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

#[cfg(target_os = "macos")]
fn parse_focused_workspace_bridge(content: String) -> Option<String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(value) = trimmed.strip_prefix("FOCUSED_WORKSPACE=") {
        let value = value.trim();
        if value.is_empty() { None } else { Some(value.to_owned()) }
    } else {
        Some(trimmed.to_owned())
    }
}

#[cfg(target_os = "macos")]
fn parse_mode_bridge(content: String) -> Option<String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }
    for key in ["MODE=", "AEROSPACE_MODE="] {
        if let Some(value) = trimmed.strip_prefix(key) {
            let value = value.trim();
            return if value.is_empty() { None } else { Some(value.to_owned()) };
        }
    }
    Some(trimmed.to_owned())
}

#[cfg(target_os = "macos")]
fn unique_preserve_order(input: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    input
        .into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
}

fn unique_sorted_workspaces(input: Vec<String>) -> Vec<String> {
    let mut output: Vec<String> = input
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    output.sort_by(|a, b| {
        match (a.parse::<i64>().ok(), b.parse::<i64>().ok()) {
            (Some(l), Some(r)) => l.cmp(&r),
            _ => a.cmp(b),
        }
    });
    output
}
