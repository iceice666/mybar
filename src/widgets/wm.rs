//! WM widget: mode badge + workspace pills with app icons.

use skia_safe::textlayout::FontCollection;
use skia_safe::{Canvas, Paint, PaintStyle, Rect};

use crate::data::BarData;
use crate::render::{draw_pill, draw_text, measure_text, text_height};
use crate::style;

include!(concat!(env!("OUT_DIR"), "/icon_map.rs"));

fn measure_icon_width(fc: &FontCollection, icon: &str) -> f32 {
    measure_text(fc, icon, style::FONT_FAMILY_ICON, style::FONT_SIZE_LG) - style::ICON_ADVANCE_TRIM
}

// -- Mode badge ---------------------------------------------------------------

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

// -- Workspace pills ----------------------------------------------------------

pub fn measure_workspaces_grouped(fc: &FontCollection, data: &BarData) -> f32 {
    if !data.wm.monitor_groups.is_empty() {
        let focused = data.wm.focused_workspace.as_deref();
        let mut total = 0.0_f32;

        for (group_index, group) in data.wm.monitor_groups.iter().enumerate() {
            if group_index > 0 {
                total += style::MONITOR_DIVIDER_MARGIN * 2.0 + style::MONITOR_DIVIDER_WIDTH;
            }

            for (workspace_index, workspace) in group.workspaces.iter().enumerate() {
                if workspace_index > 0 {
                    total += style::INNER_SPACING;
                }
                total += workspace_pill_width(fc, data, workspace, focused);
            }
        }

        return total;
    }

    measure_workspaces_flat(fc, data)
}

fn measure_workspaces_flat(fc: &FontCollection, data: &BarData) -> f32 {
    if data.wm.used_workspaces.is_empty() {
        return measure_text(fc, "--", style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM);
    }

    let focused = data.wm.focused_workspace.as_deref();
    let mut total = 0.0_f32;

    for (i, ws) in data.wm.used_workspaces.iter().enumerate() {
        if i > 0 {
            total += style::INNER_SPACING;
        }
        total += workspace_pill_width(fc, data, ws, focused);
    }

    total
}

pub fn draw_workspaces_grouped(canvas: &Canvas, fc: &FontCollection, data: &BarData, rect: Rect) {
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

    if !data.wm.monitor_groups.is_empty() {
        let focused = data.wm.focused_workspace.as_deref();
        let mut x = rect.left;

        for (group_index, group) in data.wm.monitor_groups.iter().enumerate() {
            if group_index > 0 {
                x += style::MONITOR_DIVIDER_MARGIN;
                draw_group_divider(canvas, rect, x);
                x += style::MONITOR_DIVIDER_WIDTH + style::MONITOR_DIVIDER_MARGIN;
            }

            for (workspace_index, workspace) in group.workspaces.iter().enumerate() {
                if workspace_index > 0 {
                    x += style::INNER_SPACING;
                }

                let pill_w = workspace_pill_width(fc, data, workspace, focused);
                let pill_rect = Rect::from_xywh(x, rect.top, pill_w, rect.height());
                draw_workspace_pill(canvas, fc, data, workspace, focused, pill_rect);
                x += pill_w;
            }
        }

        return;
    }

    let focused = data.wm.focused_workspace.as_deref();
    let mut x = rect.left;

    for (i, ws) in data.wm.used_workspaces.iter().enumerate() {
        if i > 0 {
            x += style::INNER_SPACING;
        }

        let pill_w = workspace_pill_width(fc, data, ws, focused);
        let pill_rect = Rect::from_xywh(x, rect.top, pill_w, rect.height());
        draw_workspace_pill(canvas, fc, data, ws, focused, pill_rect);
        x += pill_w;
    }
}

fn workspace_pill_inner_width(
    fc: &FontCollection,
    data: &BarData,
    workspace: &str,
    focused: Option<&str>,
) -> f32 {
    let active = Some(workspace) == focused;
    let mut inner = measure_text(fc, workspace, style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM);

    if active {
        for app in &data.wm.apps_in_focused_workspace {
            let icon = app_name_to_icon(app);
            inner += style::INNER_SPACING;
            if icon == ":default:" {
                inner += measure_text(fc, app, style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM);
            } else {
                inner += measure_icon_width(fc, icon);
            }
        }
    }

    inner
}

fn workspace_pill_width(
    fc: &FontCollection,
    data: &BarData,
    workspace: &str,
    focused: Option<&str>,
) -> f32 {
    workspace_pill_inner_width(fc, data, workspace, focused) + style::WIDGET_PADDING_H * 2.0
}

fn draw_workspace_pill(
    canvas: &Canvas,
    fc: &FontCollection,
    data: &BarData,
    workspace: &str,
    focused: Option<&str>,
    pill_rect: Rect,
) {
    let active = Some(workspace) == focused;
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

    let text_h = text_height(fc, workspace, style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM);
    let text_y = pill_rect.top + (pill_rect.height() - text_h) / 2.0;
    let mut tx = pill_rect.left + style::WIDGET_PADDING_H;
    draw_text(
        canvas,
        fc,
        workspace,
        tx,
        text_y,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_SM,
        style::TEXT_COLOR,
    );
    tx += measure_text(fc, workspace, style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM);

    if active {
        for app in &data.wm.apps_in_focused_workspace {
            tx += style::INNER_SPACING;
            let icon = app_name_to_icon(app);
            if icon == ":default:" {
                let ih = text_height(fc, app, style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM);
                let iy = pill_rect.top + (pill_rect.height() - ih) / 2.0;
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
                let iy = pill_rect.top + (pill_rect.height() - ih) / 2.0;
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
}

fn draw_group_divider(canvas: &Canvas, rect: Rect, x: f32) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(PaintStyle::Fill);
    paint.set_color4f(style::MONITOR_DIVIDER_COLOR, None);

    let h = (rect.height() - 8.0).max(0.0);
    let y = rect.top + (rect.height() - h) / 2.0;
    let divider = Rect::from_xywh(x, y, style::MONITOR_DIVIDER_WIDTH, h);
    canvas.draw_rect(divider, &paint);
}
