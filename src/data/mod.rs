//! Data collection: all widget state gathered by tokio tasks, exposed via watch channels.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::watch;

mod wm_bridge;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux::*;

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
    pub wifi_signal: Option<u8>,
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
            wifi_signal: None,
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

    // WM bridge listener (platform-agnostic) – direct push over Unix socket
    {
        let tx = tx.clone();
        let notifier = notifier.clone();
        let app_loader: wm_bridge::AppLoader =
            Arc::new(|ws: String| Box::pin(async move { load_apps_for_workspace(&ws).await }));
        rt.spawn(async move {
            wm_bridge::run_wm_bridge_listener(tx, notifier, app_loader).await;
        });
    }

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
                let time = now.format("%H:%M:%S").to_string();
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

                let wifi_signal = tokio::task::spawn_blocking(|| read_wifi_signal())
                    .await
                    .ok()
                    .flatten();

                tx.send_modify(|d| {
                    d.net_upload = smooth_up;
                    d.net_download = smooth_down;
                    d.wifi_signal = wifi_signal;
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

    // WM (AeroSpace on macOS, stubs on Linux) – full refresh 10s
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
            let mut slow = tokio::time::interval(Duration::from_secs(10));

            loop {
                slow.tick().await;
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

pub(crate) fn unique_sorted_workspaces(input: Vec<String>) -> Vec<String> {
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
