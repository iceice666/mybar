use crate::FONT_ICON;
use iced::Element;
use iced::widget::{Row, button, text};

include!(concat!(env!("OUT_DIR"), "/icon_map.rs"));

const FOCUSED_WORKSPACE_BRIDGE_PATH: &str = "/tmp/mybar-aerospace-focused-workspace";
const MODE_BRIDGE_PATH: &str = "/tmp/aerospace-mode";

#[derive(Debug, Clone)]
pub struct Data {
    pub mode: String,
    pub used_workspaces: Vec<String>,
    pub focused_workspace: Option<String>,
    pub apps_in_focused_workspace: Vec<String>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            mode: String::from("main"),
            used_workspaces: Vec::new(),
            focused_workspace: None,
            apps_in_focused_workspace: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct State {
    data: Data,
}

impl State {
    pub fn apply(&mut self, data: Data) {
        self.data = data;
    }

    pub fn set_focused_workspace(&mut self, focused_workspace: Option<String>) -> bool {
        if self.data.focused_workspace == focused_workspace {
            return false;
        }

        self.data.focused_workspace = focused_workspace.clone();
        if let Some(workspace) = focused_workspace {
            if !self.data.used_workspaces.iter().any(|ws| ws == &workspace) {
                self.data.used_workspaces.push(workspace);
                self.data.used_workspaces =
                    unique_sorted_workspaces(self.data.used_workspaces.clone());
            }
        }
        true
    }

    pub fn set_apps_in_focused_workspace(&mut self, apps: Vec<String>) {
        self.data.apps_in_focused_workspace = apps;
    }

    pub fn set_mode(&mut self, mode: Option<String>) {
        if let Some(mode) = mode {
            self.data.mode = mode;
        }
    }

    pub fn view_mode<'a>(&'a self) -> Element<'a, crate::Message> {
        button(text(format!("{}:", self.data.mode)).color(crate::hex!(0x000000)))
            .padding([1.0, 6.0])
            .style(crate::widget_container_style)
            .into()
    }

    pub fn view_workspaces<'a>(&'a self) -> Element<'a, crate::Message> {
        if self.data.used_workspaces.is_empty() {
            return text(String::from("--")).color(crate::hex!(0x000000)).into();
        }

        let focused = self.data.focused_workspace.as_deref();
        let row = self.data.used_workspaces.iter().fold(
            Row::new().spacing(4).align_y(iced::Alignment::Center),
            |row, ws| {
                let _active = Some(ws.as_str()) == focused;
                let badge = button(text(ws).color(crate::hex!(0x000000)))
                    .padding([1.0, 6.0])
                    .style(crate::widget_container_style);
                row.push(badge)
            },
        );

        row.into()
    }

    pub fn view_apps<'a>(&'a self) -> Element<'a, crate::Message> {
        if self.data.apps_in_focused_workspace.is_empty() {
            return text(String::from("")).into();
        }

        let row = self.data.apps_in_focused_workspace.iter().fold(
            Row::new().spacing(4).align_y(iced::Alignment::Center),
            |row, app| {
                let icon = app_name_to_icon(app);
                row.push(
                    text(icon)
                        .font(FONT_ICON)
                        .size(18)
                        .color(crate::hex!(0x000000)),
                )
            },
        );

        row.into()
    }
}

#[cfg(target_os = "macos")]
pub async fn load_data() -> Data {
    Data {
        mode: mode().unwrap_or_else(|| String::from("main")),
        used_workspaces: used_workspaces(),
        focused_workspace: focused_workspace(),
        apps_in_focused_workspace: apps_in_focused_workspace(),
    }
}

#[cfg(not(target_os = "macos"))]
pub async fn load_data() -> Data {
    Data::default()
}

#[cfg(target_os = "macos")]
pub async fn load_focused_workspace_from_bridge() -> Option<String> {
    parse_focused_workspace_bridge(std::fs::read_to_string(FOCUSED_WORKSPACE_BRIDGE_PATH).ok()?)
}

#[cfg(not(target_os = "macos"))]
pub async fn load_focused_workspace_from_bridge() -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
pub async fn load_mode_from_bridge() -> Option<String> {
    parse_mode_bridge(std::fs::read_to_string(MODE_BRIDGE_PATH).ok()?)
}

#[cfg(not(target_os = "macos"))]
pub async fn load_mode_from_bridge() -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
pub async fn load_apps_for_workspace(workspace: Option<String>) -> Vec<String> {
    let Some(workspace) = workspace else {
        return Vec::new();
    };

    let preferred = run_aerospace(&[
        "list-windows",
        "--workspace",
        workspace.as_str(),
        "--format",
        "%{app-name}",
    ])
    .map(parse_lines)
    .unwrap_or_default();
    if !preferred.is_empty() {
        return unique_preserve_order(preferred);
    }

    let fallback = run_aerospace(&["list-windows", "--workspace", workspace.as_str()])
        .map(parse_lines)
        .unwrap_or_default();
    let parsed = fallback
        .iter()
        .filter_map(|line| line.split('|').nth(1))
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    unique_preserve_order(parsed)
}

#[cfg(not(target_os = "macos"))]
pub async fn load_apps_for_workspace(_workspace: Option<String>) -> Vec<String> {
    Vec::new()
}

#[cfg(target_os = "macos")]
fn mode() -> Option<String> {
    std::fs::read_to_string(MODE_BRIDGE_PATH)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[cfg(target_os = "macos")]
fn used_workspaces() -> Vec<String> {
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
fn focused_workspace() -> Option<String> {
    run_aerospace(&["list-workspaces", "--focused"])
        .and_then(|output| parse_lines(output).into_iter().next())
}

#[cfg(target_os = "macos")]
fn apps_in_focused_workspace() -> Vec<String> {
    let preferred = run_aerospace(&[
        "list-windows",
        "--workspace",
        "focused",
        "--format",
        "%{app-name}",
    ])
    .map(parse_lines)
    .unwrap_or_default();
    if !preferred.is_empty() {
        return unique_preserve_order(preferred);
    }

    let fallback = run_aerospace(&["list-windows", "--workspace", "focused"])
        .map(parse_lines)
        .unwrap_or_default();
    let parsed = fallback
        .iter()
        .filter_map(|line| line.split('|').nth(1))
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    unique_preserve_order(parsed)
}

#[cfg(target_os = "macos")]
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

#[cfg(target_os = "macos")]
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

#[cfg(target_os = "macos")]
fn run_aerospace(args: &[&str]) -> Option<String> {
    use std::process::Command;

    let output = Command::new("aerospace").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let content = String::from_utf8(output.stdout).ok()?;
    Some(content.trim().to_owned())
}

#[cfg(target_os = "macos")]
fn parse_lines(output: String) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
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
    output.sort_by(|left, right| {
        let left_num = left.parse::<i64>().ok();
        let right_num = right.parse::<i64>().ok();
        match (left_num, right_num) {
            (Some(l), Some(r)) => l.cmp(&r),
            _ => left.cmp(right),
        }
    });
    output
}
