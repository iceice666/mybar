//! WM widget: mode badge + workspace pills with app icons.

use skia_safe::textlayout::FontCollection;
use skia_safe::{Canvas, Rect};

use crate::data::BarData;
use crate::render::{draw_pill, draw_text, measure_text, text_height};
use crate::style;

include!(concat!(env!("OUT_DIR"), "/icon_map.rs"));

fn measure_icon_width(fc: &FontCollection, icon: &str) -> f32 {
    measure_text(fc, icon, style::FONT_FAMILY_ICON, style::FONT_SIZE_LG) - style::ICON_ADVANCE_TRIM
}

// ── Mode badge ───────────────────────────────────────────────────────────────

pub fn measure_mode(fc: &FontCollection, data: &BarData) -> f32 {
    let label = format!("{}:", data.wm.mode);
    let w = measure_text(fc, &label, style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM);
    w + style::WIDGET_PADDING_H * 2.0
}

pub fn draw_mode(canvas: &Canvas, fc: &FontCollection, data: &BarData, rect: Rect) {
    draw_pill(
        canvas,
        rect,
        style::WIDGET_RADIUS,
        style::WIDGET_BG,
        style::WIDGET_BORDER,
        style::WIDGET_BORDER_WIDTH,
    );

    let label = format!("{}:", data.wm.mode);
    let h = text_height(fc, &label, style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM);
    let y = rect.top + (rect.height() - h) / 2.0;
    let x = rect.left + style::WIDGET_PADDING_H;

    draw_text(
        canvas,
        fc,
        &label,
        x,
        y,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_SM,
        style::TEXT_COLOR,
    );
}

// ── Workspace pills ──────────────────────────────────────────────────────────

/// Measure total width of all workspace pills.
pub fn measure_workspaces(fc: &FontCollection, data: &BarData) -> f32 {
    if data.wm.used_workspaces.is_empty() {
        return measure_text(fc, "--", style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM);
    }

    let focused = data.wm.focused_workspace.as_deref();
    let mut total = 0.0_f32;

    for (i, ws) in data.wm.used_workspaces.iter().enumerate() {
        if i > 0 {
            total += style::INNER_SPACING;
        }
        let active = Some(ws.as_str()) == focused;
        let mut pill_inner = measure_text(fc, ws, style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM);

        if active {
            for app in &data.wm.apps_in_focused_workspace {
                let icon = app_name_to_icon(app);
                pill_inner += style::INNER_SPACING;
                if icon == ":default:" {
                    pill_inner +=
                        measure_text(fc, app, style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM);
                } else {
                    pill_inner += measure_icon_width(fc, icon);
                }
            }
        }

        total += pill_inner + style::WIDGET_PADDING_H * 2.0;
    }

    total
}

/// Draw all workspace pills into the given rect.
pub fn draw_workspaces(canvas: &Canvas, fc: &FontCollection, data: &BarData, rect: Rect) {
    if data.wm.used_workspaces.is_empty() {
        let h = text_height(fc, "--", style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM);
        let y = rect.top + (rect.height() - h) / 2.0;
        draw_text(
            canvas,
            fc,
            "--",
            rect.left,
            y,
            style::FONT_FAMILY_TEXT,
            style::FONT_SIZE_SM,
            style::TEXT_COLOR,
        );
        return;
    }

    let focused = data.wm.focused_workspace.as_deref();
    let mut x = rect.left;

    for (i, ws) in data.wm.used_workspaces.iter().enumerate() {
        if i > 0 {
            x += style::INNER_SPACING;
        }

        let active = Some(ws.as_str()) == focused;

        // Compute pill width
        let mut pill_inner = measure_text(fc, ws, style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM);
        if active {
            for app in &data.wm.apps_in_focused_workspace {
                let icon = app_name_to_icon(app);
                pill_inner += style::INNER_SPACING;
                if icon == ":default:" {
                    pill_inner +=
                        measure_text(fc, app, style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM);
                } else {
                    pill_inner += measure_icon_width(fc, icon);
                }
            }
        }

        let pill_w = pill_inner + style::WIDGET_PADDING_H * 2.0;
        let pill_rect = Rect::from_xywh(x, rect.top, pill_w, rect.height());

        let (bg, border) = if active {
            (style::FOCUSED_BG, style::FOCUSED_BORDER)
        } else {
            (style::WIDGET_BG, style::WIDGET_BORDER)
        };
        draw_pill(
            canvas,
            pill_rect,
            style::WIDGET_RADIUS,
            bg,
            border,
            style::WIDGET_BORDER_WIDTH,
        );

        // Draw workspace name
        let text_h = text_height(fc, ws, style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM);
        let text_y = rect.top + (rect.height() - text_h) / 2.0;
        let mut tx = x + style::WIDGET_PADDING_H;
        draw_text(
            canvas,
            fc,
            ws,
            tx,
            text_y,
            style::FONT_FAMILY_TEXT,
            style::FONT_SIZE_SM,
            style::TEXT_COLOR,
        );
        tx += measure_text(fc, ws, style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM);

        // Draw app icons for focused workspace
        if active {
            for app in &data.wm.apps_in_focused_workspace {
                tx += style::INNER_SPACING;
                let icon = app_name_to_icon(app);
                if icon == ":default:" {
                    let ih = text_height(fc, app, style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM);
                    let iy = rect.top + (rect.height() - ih) / 2.0;
                    draw_text(
                        canvas,
                        fc,
                        app,
                        tx,
                        iy,
                        style::FONT_FAMILY_TEXT,
                        style::FONT_SIZE_SM,
                        style::TEXT_COLOR,
                    );
                    tx += measure_text(fc, app, style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM);
                } else {
                    let ih = text_height(fc, icon, style::FONT_FAMILY_ICON, style::FONT_SIZE_LG);
                    let iy = rect.top + (rect.height() - ih) / 2.0;
                    draw_text(
                        canvas,
                        fc,
                        icon,
                        tx,
                        iy,
                        style::FONT_FAMILY_ICON,
                        style::FONT_SIZE_LG,
                        style::TEXT_COLOR,
                    );
                    tx += measure_icon_width(fc, icon);
                }
            }
        }

        x += pill_w;
    }
}
