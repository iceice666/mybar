//! Network widget: up/down throughput with arrows icon in a pill.

use skia_safe::textlayout::FontCollection;
use skia_safe::{Canvas, Rect};

use crate::data::BarData;
use crate::render::{draw_pill, draw_text, measure_text, text_height};
use crate::style;

// Arrow up-down icon from Nerd Font (Cascadia Code NF)
const ICON_ARROW_UP_DOWN: &str = "\u{F07D1}";

fn format_k(bytes_per_sec: f64) -> String {
    let kb = bytes_per_sec / 1024.0;
    if kb < 1.0 {
        return " -- ".to_string();
    }
    if kb < 1024.0 {
        if kb < 100.0 {
            format!("{:.1}K", kb)
        } else {
            format!("{:.0}K ", kb)
        }
    } else {
        let mb = kb / 1024.0;
        if mb < 100.0 {
            format!("{:.1}M", mb)
        } else {
            format!("{:.0}M ", mb)
        }
    }
}

pub fn measure(fc: &FontCollection, data: &BarData) -> f32 {
    let icon_w = measure_text(
        fc,
        ICON_ARROW_UP_DOWN,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_TIME,
    );

    let up_label_w = measure_text(fc, "UP", style::FONT_FAMILY_TEXT, style::FONT_SIZE_LABEL);
    let up_val = format_k(data.net_upload);
    let up_val_w = measure_text(fc, &up_val, style::FONT_FAMILY_TEXT, style::FONT_SIZE_TIME);
    let up_w = up_label_w.max(up_val_w);

    let dn_label_w = measure_text(fc, "DOWN", style::FONT_FAMILY_TEXT, style::FONT_SIZE_LABEL);
    let dn_val = format_k(data.net_download);
    let dn_val_w = measure_text(fc, &dn_val, style::FONT_FAMILY_TEXT, style::FONT_SIZE_TIME);
    let dn_w = dn_label_w.max(dn_val_w);

    icon_w
        + style::INNER_SPACING
        + up_w
        + style::INNER_SPACING
        + dn_w
        + style::WIDGET_PADDING_H * 2.0
}

pub fn draw(canvas: &Canvas, fc: &FontCollection, data: &BarData, rect: Rect) {
    draw_pill(
        canvas,
        rect,
        style::WIDGET_RADIUS,
        style::WIDGET_BG,
        style::WIDGET_BORDER,
        style::WIDGET_BORDER_WIDTH,
    );

    let mut x = rect.left + style::WIDGET_PADDING_H;

    // Icon (vertically centered)
    let icon_h = text_height(
        fc,
        ICON_ARROW_UP_DOWN,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_TIME,
    );
    let icon_y = rect.top + (rect.height() - icon_h) / 2.0;
    draw_text(
        canvas,
        fc,
        ICON_ARROW_UP_DOWN,
        x,
        icon_y,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_TIME,
        style::TEXT_COLOR,
    );
    let icon_w = measure_text(
        fc,
        ICON_ARROW_UP_DOWN,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_TIME,
    );
    x += icon_w + style::INNER_SPACING;

    // Upload column (label + value stacked)
    let label_h = text_height(fc, "UP", style::FONT_FAMILY_TEXT, style::FONT_SIZE_LABEL);
    let val_h = text_height(fc, "0K", style::FONT_FAMILY_TEXT, style::FONT_SIZE_TIME);
    let col_h = label_h + val_h;
    let col_y = rect.top + (rect.height() - col_h) / 2.0;

    let up_label_w = measure_text(fc, "UP", style::FONT_FAMILY_TEXT, style::FONT_SIZE_LABEL);
    let up_val = format_k(data.net_upload);
    let up_val_w = measure_text(fc, &up_val, style::FONT_FAMILY_TEXT, style::FONT_SIZE_TIME);
    let up_col_w = up_label_w.max(up_val_w);

    // Center label within column
    let up_label_x = x + (up_col_w - up_label_w) / 2.0;
    draw_text(
        canvas,
        fc,
        "UP",
        up_label_x,
        col_y,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_LABEL,
        style::TEXT_COLOR,
    );
    // Center value within column
    let up_val_x = x + (up_col_w - up_val_w) / 2.0;
    draw_text(
        canvas,
        fc,
        &up_val,
        up_val_x,
        col_y + label_h,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_TIME,
        style::TEXT_COLOR,
    );

    x += up_col_w + style::INNER_SPACING;

    // Download column
    let dn_label_w = measure_text(fc, "DOWN", style::FONT_FAMILY_TEXT, style::FONT_SIZE_LABEL);
    let dn_val = format_k(data.net_download);
    let dn_val_w = measure_text(fc, &dn_val, style::FONT_FAMILY_TEXT, style::FONT_SIZE_TIME);
    let dn_col_w = dn_label_w.max(dn_val_w);

    // Center label within column
    let dn_label_x = x + (dn_col_w - dn_label_w) / 2.0;
    draw_text(
        canvas,
        fc,
        "DOWN",
        dn_label_x,
        col_y,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_LABEL,
        style::TEXT_COLOR,
    );
    // Center value within column
    let dn_val_x = x + (dn_col_w - dn_val_w) / 2.0;
    draw_text(
        canvas,
        fc,
        &dn_val,
        dn_val_x,
        col_y + label_h,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_TIME,
        style::TEXT_COLOR,
    );
}
