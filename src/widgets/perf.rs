//! Performance widget: CPU icon + CPU% + MEM columns in a pill.

use skia_safe::textlayout::FontCollection;
use skia_safe::{Canvas, Rect};

use crate::data::BarData;
use crate::render::{draw_pill, draw_text, measure_text, text_height};
use crate::style;

// CPU icon from Nerd Font (Cascadia Code NF)
const ICON_CPU: &str = "\u{F0EE0}";

fn convert_to_gb(bytes: u64) -> f64 {
    bytes as f64 / 1024.0 / 1024.0 / 1024.0
}

pub fn measure(fc: &FontCollection, data: &BarData) -> f32 {
    let icon_w = measure_text(
        fc,
        ICON_CPU,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_DEFAULT,
    );

    let cpu_label_w = measure_text(fc, "CPU", style::FONT_FAMILY_TEXT, style::FONT_SIZE_LABEL);
    let cpu_val = format!("{:>3.0}%", data.cpu_percent);
    let cpu_val_w = measure_text(
        fc,
        &cpu_val,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_DEFAULT,
    );
    let cpu_w = cpu_label_w.max(cpu_val_w);

    let mem_label = format!("MEM /{}G", convert_to_gb(data.mem_total));
    let mem_label_w = measure_text(
        fc,
        &mem_label,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_LABEL,
    );
    let mem_val = format!("{:.1}G", convert_to_gb(data.mem_used));
    let mem_val_w = measure_text(
        fc,
        &mem_val,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_DEFAULT,
    );
    let mem_w = mem_label_w.max(mem_val_w);

    icon_w
        + style::INNER_SPACING
        + cpu_w
        + style::INNER_SPACING
        + mem_w
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

    // CPU icon
    let icon_h = text_height(
        fc,
        ICON_CPU,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_DEFAULT,
    );
    let icon_y = rect.top + (rect.height() - icon_h) / 2.0;
    draw_text(
        canvas,
        fc,
        ICON_CPU,
        x,
        icon_y,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_DEFAULT,
        style::TEXT_COLOR,
    );
    let icon_w = measure_text(
        fc,
        ICON_CPU,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_DEFAULT,
    );
    x += icon_w + style::INNER_SPACING;

    // CPU column
    let label_h = text_height(fc, "CPU", style::FONT_FAMILY_TEXT, style::FONT_SIZE_LABEL);
    let val_h = text_height(fc, "0%", style::FONT_FAMILY_TEXT, style::FONT_SIZE_DEFAULT);
    let col_h = label_h + val_h;
    let col_y = rect.top + (rect.height() - col_h) / 2.0;

    let cpu_label_w = measure_text(fc, "CPU", style::FONT_FAMILY_TEXT, style::FONT_SIZE_LABEL);
    let cpu_val = format!("{:>3.0}%", data.cpu_percent);
    let cpu_val_w = measure_text(
        fc,
        &cpu_val,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_DEFAULT,
    );
    let cpu_col_w = cpu_label_w.max(cpu_val_w);

    // Center label within column
    let cpu_label_x = x + (cpu_col_w - cpu_label_w) / 2.0;
    draw_text(
        canvas,
        fc,
        "CPU",
        cpu_label_x,
        col_y,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_LABEL,
        style::TEXT_COLOR,
    );
    // Center value within column
    let cpu_val_x = x + (cpu_col_w - cpu_val_w) / 2.0;
    draw_text(
        canvas,
        fc,
        &cpu_val,
        cpu_val_x,
        col_y + label_h,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_DEFAULT,
        style::TEXT_COLOR,
    );

    x += cpu_col_w + style::INNER_SPACING;

    // MEM column
    let mem_label = format!("MEM /{}G", convert_to_gb(data.mem_total));
    let mem_label_w = measure_text(
        fc,
        &mem_label,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_LABEL,
    );
    let mem_val = format!("{:.1}G", convert_to_gb(data.mem_used));
    let mem_val_w = measure_text(
        fc,
        &mem_val,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_DEFAULT,
    );
    let mem_col_w = mem_label_w.max(mem_val_w);

    // Center label within column
    let mem_label_x = x + (mem_col_w - mem_label_w) / 2.0;
    draw_text(
        canvas,
        fc,
        &mem_label,
        mem_label_x,
        col_y,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_LABEL,
        style::TEXT_COLOR,
    );
    // Center value within column
    let mem_val_x = x + (mem_col_w - mem_val_w) / 2.0;
    draw_text(
        canvas,
        fc,
        &mem_val,
        mem_val_x,
        col_y + label_h,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_DEFAULT,
        style::TEXT_COLOR,
    );
}
