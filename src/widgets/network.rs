//! Network widget: up/down throughput with arrows icon in a pill.

use skia_safe::textlayout::FontCollection;
use skia_safe::{Canvas, Rect};

use crate::data::BarData;
use crate::render::{draw_pill, draw_text, measure_text, text_height};
use crate::style;

// Icons from Nerd Font (Cascadia Code NF)
const ICON_WIFI_UNKNOWN: &str = "\u{F092B}";
// WiFi strength 1–4 bars (Material Design Icons)
const ICON_WIFI_1: &str = "\u{F091F}";
const ICON_WIFI_2: &str = "\u{F0922}";
const ICON_WIFI_3: &str = "\u{F0925}";
const ICON_WIFI_4: &str = "\u{F0928}";

fn network_icon(wifi_signal: Option<u8>) -> &'static str {
    let Some(pct) = wifi_signal else {
        return ICON_WIFI_UNKNOWN;
    };
    match pct {
        0..=25 => ICON_WIFI_1,
        26..=50 => ICON_WIFI_2,
        51..=75 => ICON_WIFI_3,
        _ => ICON_WIFI_4,
    }
}

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
    let icon = network_icon(data.wifi_signal);
    let icon_w = measure_text(
        fc,
        icon,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_2XL,
    );

    let up_label_w = measure_text(fc, "UP", style::FONT_FAMILY_TEXT, style::FONT_SIZE_2XS);
    let up_val = format_k(data.net_upload);
    let up_val_w = measure_text(fc, &up_val, style::FONT_FAMILY_TEXT, style::FONT_SIZE_BASE);
    let up_w = up_label_w.max(up_val_w);

    let dn_label_w = measure_text(fc, "DOWN", style::FONT_FAMILY_TEXT, style::FONT_SIZE_2XS);
    let dn_val = format_k(data.net_download);
    let dn_val_w = measure_text(fc, &dn_val, style::FONT_FAMILY_TEXT, style::FONT_SIZE_BASE);
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

    // Icon (vertically centered; varies by WiFi signal when on WiFi)
    let icon = network_icon(data.wifi_signal);
    let icon_h = text_height(
        fc,
        icon,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_2XL,
    );
    let icon_y = rect.top + (rect.height() - icon_h) / 2.0;
    draw_text(
        canvas,
        fc,
        icon,
        x,
        icon_y,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_2XL,
        style::TEXT_COLOR,
    );
    let icon_w = measure_text(
        fc,
        icon,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_2XL,
    );
    x += icon_w + style::INNER_SPACING;

    // Upload column (label + value stacked)
    let label_h = text_height(fc, "UP", style::FONT_FAMILY_TEXT, style::FONT_SIZE_2XS);
    let val_h = text_height(fc, "0K", style::FONT_FAMILY_TEXT, style::FONT_SIZE_BASE);
    let col_h = label_h + val_h;
    let col_y = rect.top + (rect.height() - col_h) / 2.0;

    let up_label_w = measure_text(fc, "UP", style::FONT_FAMILY_TEXT, style::FONT_SIZE_2XS);
    let up_val = format_k(data.net_upload);
    let up_val_w = measure_text(fc, &up_val, style::FONT_FAMILY_TEXT, style::FONT_SIZE_BASE);
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
        style::FONT_SIZE_2XS,
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
        style::FONT_SIZE_BASE,
        style::TEXT_COLOR,
    );

    x += up_col_w + style::INNER_SPACING;

    // Download column
    let dn_label_w = measure_text(fc, "DOWN", style::FONT_FAMILY_TEXT, style::FONT_SIZE_2XS);
    let dn_val = format_k(data.net_download);
    let dn_val_w = measure_text(fc, &dn_val, style::FONT_FAMILY_TEXT, style::FONT_SIZE_BASE);
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
        style::FONT_SIZE_2XS,
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
        style::FONT_SIZE_BASE,
        style::TEXT_COLOR,
    );
}
