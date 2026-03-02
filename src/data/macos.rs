//! macOS-specific data sources: AeroSpace WM, nowplaying-cli, bridge files.

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

pub async fn load_focused_workspace_bridge() -> Option<String> {
    let content = std::fs::read_to_string("/tmp/mybar-aerospace-focused-workspace").ok()?;
    parse_focused_workspace_bridge(content)
}

pub async fn load_mode_bridge() -> Option<String> {
    let content = std::fs::read_to_string("/tmp/aerospace-mode").ok()?;
    parse_mode_bridge(content)
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

fn parse_focused_workspace_bridge(content: String) -> Option<String> {
    let trimmed = content.trim();
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
        Some(trimmed.to_owned())
    }
}

fn parse_mode_bridge(content: String) -> Option<String> {
    let trimmed = content.trim();
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
    Some(trimmed.to_owned())
}

fn unique_preserve_order(input: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    input
        .into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
}
