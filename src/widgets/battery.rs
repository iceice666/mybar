//! Battery widget: icon + percentage in a pill.

use skia_safe::textlayout::FontCollection;
use skia_safe::{Canvas, Rect};

use crate::data::BarData;
use crate::render::{draw_pill, draw_text, measure_text, text_height};
use crate::style;

/// Get the battery icon character (Nerd Font codepoints from Cascadia Code NF).
fn battery_icon(percent: u8, charging: bool) -> &'static str {
    if charging {
        match percent {
            91.. => "\u{F0085}",
            81..91 => "\u{F008B}",
            71..81 => "\u{F008A}",
            61..71 => "\u{F089E}",
            51..61 => "\u{F0089}",
            41..51 => "\u{F089D}",
            31..41 => "\u{F0088}",
            21..31 => "\u{F0087}",
            11..21 => "\u{F0086}",
            0..11 => "\u{F089C}",
        }
    } else {
        match percent {
            91.. => "\u{F0079}",
            81..91 => "\u{F0082}",
            71..81 => "\u{F0081}",
            61..71 => "\u{F0080}",
            51..61 => "\u{F007F}",
            41..51 => "\u{F007E}",
            31..41 => "\u{F007D}",
            21..31 => "\u{F007C}",
            11..21 => "\u{F007B}",
            0..11 => "\u{F007A}",
        }
    }
}

pub fn measure(fc: &FontCollection, data: &BarData) -> f32 {
    let Some(pct) = data.battery_percent else {
        return 0.0;
    };
    let icon = battery_icon(pct, data.battery_charging);
    let icon_w = measure_text(
        fc,
        icon,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_2XL,
    );
    let label = format!("{pct}%");
    let label_w = measure_text(
        fc,
        &label,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_SM,
    );
    icon_w + style::INNER_SPACING + label_w + style::WIDGET_PADDING_H * 2.0
}

pub fn draw(canvas: &Canvas, fc: &FontCollection, data: &BarData, rect: Rect) {
    let Some(pct) = data.battery_percent else {
        return;
    };

    draw_pill(
        canvas,
        rect,
        style::WIDGET_RADIUS,
        style::WIDGET_BG,
        style::WIDGET_BORDER,
        style::WIDGET_BORDER_WIDTH,
    );

    let icon = battery_icon(pct, data.battery_charging);
    let label = format!("{pct}%");

    let icon_h = text_height(
        fc,
        icon,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_2XL,
    );
    let label_h = text_height(
        fc,
        &label,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_SM,
    );

    let x = rect.left + style::WIDGET_PADDING_H;

    // Vertically center icon
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

    // Vertically center label
    let label_y = rect.top + (rect.height() - label_h) / 2.0;
    draw_text(
        canvas,
        fc,
        &label,
        x + icon_w + style::INNER_SPACING,
        label_y,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_SM,
        style::TEXT_COLOR,
    );
}
