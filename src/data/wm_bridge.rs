use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::task;

use super::{BarData, RedrawNotifier, WmData};

/// Type alias for an async workspace app loader.
pub type AppLoader =
    Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = Vec<String>> + Send>> + Send + Sync>;

/// Type alias for an async WM snapshot loader.
pub type WmLoader = Arc<dyn Fn() -> Pin<Box<dyn Future<Output = WmData> + Send>> + Send + Sync>;

const WM_BRIDGE_SOCKET: &str = "/tmp/mybar-wm-bridge.sock";

/// Run a Unix domain socket listener that accepts pushed WM updates.
///
/// Protocol (per line):
/// - `MODE=<value>` or `AEROSPACE_MODE=<value>` updates `wm.mode`
/// - `FOCUSED_WORKSPACE=<value>` or bare `<value>` updates `wm.focused_workspace`
/// - `UPDATE_ALL` force-refreshes all WM state (mode/workspaces/focused/apps)
pub async fn run_wm_bridge_listener(
    tx: Arc<tokio::sync::watch::Sender<BarData>>,
    notifier: RedrawNotifier,
    app_loader: AppLoader,
    wm_loader: WmLoader,
) {
    // Remove any stale socket from previous run.
    let _ = std::fs::remove_file(WM_BRIDGE_SOCKET);

    let listener = match UnixListener::bind(WM_BRIDGE_SOCKET) {
        Ok(l) => l,
        Err(err) => {
            crate::logging::error(&format!(
                "wm_bridge: failed to bind {}: {}",
                WM_BRIDGE_SOCKET, err
            ));
            return;
        }
    };

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(pair) => pair,
            Err(err) => {
                crate::logging::error(&format!("wm_bridge: accept error: {}", err));
                continue;
            }
        };

        let tx = tx.clone();
        let notifier = notifier.clone();
        let app_loader = app_loader.clone();
        let wm_loader = wm_loader.clone();
        task::spawn(async move {
            if let Err(err) = handle_stream(stream, tx, notifier, app_loader, wm_loader).await {
                crate::logging::error(&format!("wm_bridge: stream handler error: {}", err));
            }
        });
    }
}

async fn handle_stream(
    stream: UnixStream,
    tx: Arc<tokio::sync::watch::Sender<BarData>>,
    notifier: RedrawNotifier,
    app_loader: AppLoader,
    wm_loader: WmLoader,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Mode update takes precedence when explicitly tagged.
        if let Some(mode) = parse_mode_line(trimmed) {
            apply_mode_update(&tx, &notifier, mode);
            continue;
        }

        if parse_update_all_line(trimmed) {
            apply_force_update(&tx, &notifier, &app_loader, &wm_loader).await;
            continue;
        }

        if let Some(ws) = parse_focused_workspace_line(trimmed) {
            apply_workspace_update(&tx, &notifier, &app_loader, ws).await;
        }
    }

    Ok(())
}

fn parse_update_all_line(line: &str) -> bool {
    line.trim() == "UPDATE_ALL"
}

fn parse_mode_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    for key in ["MODE=", "AEROSPACE_MODE="] {
        if let Some(value) = trimmed.strip_prefix(key) {
            let value = value.trim();
            return if value.is_empty() {
                None
            } else {
                Some(value.to_owned())
            };
        }
    }
    None
}

fn parse_focused_workspace_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(value) = trimmed.strip_prefix("FOCUSED_WORKSPACE=") {
        let value = value.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_owned())
        }
    } else {
        // Bare value: treat as workspace name.
        Some(trimmed.to_owned())
    }
}

fn apply_mode_update(
    tx: &Arc<tokio::sync::watch::Sender<BarData>>,
    notifier: &RedrawNotifier,
    mode: String,
) {
    let mut changed = false;
    tx.send_modify(|d| {
        if d.wm.mode != mode {
            d.wm.mode = mode.clone();
            changed = true;
        }
    });
    if changed {
        notifier();
    }
}

async fn apply_workspace_update(
    tx: &Arc<tokio::sync::watch::Sender<BarData>>,
    notifier: &RedrawNotifier,
    app_loader: &AppLoader,
    ws: String,
) {
    let prev_focused = tx.borrow().wm.focused_workspace.clone();

    let mut changed = false;
    let mut focused_changed = false;
    let mut new_focused: Option<String> = None;

    tx.send_modify(|d| {
        if !d.wm.used_workspaces.iter().any(|w| w == &ws) {
            d.wm.used_workspaces.push(ws.clone());
            d.wm.used_workspaces =
                super::unique_sorted_workspaces(std::mem::take(&mut d.wm.used_workspaces));
            changed = true;
        }

        if !d.wm.monitor_groups.is_empty() {
            for group in d.wm.monitor_groups.iter_mut() {
                if !group.workspaces.iter().any(|w| w == &ws) {
                    group.workspaces.push(ws.clone());
                    group.workspaces = super::unique_sorted_workspaces(std::mem::take(&mut group.workspaces));
                    changed = true;
                }
            }
        }

        if d.wm.focused_workspace.as_deref() != Some(ws.as_str()) {
            d.wm.focused_workspace = Some(ws.clone());
            changed = true;
            focused_changed = true;
            new_focused = Some(ws.clone());
        }
    });

    if focused_changed {
        if let Some(ref focused_ws) = new_focused {
            if prev_focused.as_deref() != Some(focused_ws.as_str()) {
                let apps = (app_loader)(focused_ws.clone()).await;
                tx.send_modify(|d| d.wm.apps_in_focused_workspace = apps);
            }
        }
    }

    if changed {
        notifier();
    }
}

async fn apply_force_update(
    tx: &Arc<tokio::sync::watch::Sender<BarData>>,
    notifier: &RedrawNotifier,
    app_loader: &AppLoader,
    wm_loader: &WmLoader,
) {
    let mut wm = (wm_loader)().await;
    if let Some(focused_ws) = wm.focused_workspace.clone() {
        let apps = (app_loader)(focused_ws).await;
        wm.apps_in_focused_workspace = apps;
    }

    tx.send_modify(|d| d.wm = wm.clone());
    notifier();
}
