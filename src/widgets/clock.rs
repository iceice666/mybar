//! Clock widget: date (small) above time (larger) in a pill.

use skia_safe::textlayout::FontCollection;
use skia_safe::{Canvas, Rect};

use crate::data::BarData;
use crate::render::{draw_pill, draw_text, measure_text, text_height};
use crate::style;

/// Measure the width this widget needs.
pub fn measure(fc: &FontCollection, data: &BarData) -> f32 {
    let date_w = measure_text(
        fc,
        &data.date,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_XS,
    );
    let time_w = measure_text(
        fc,
        &data.time,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_BASE,
    );
    let inner = date_w.max(time_w);
    inner + style::WIDGET_PADDING_H * 2.0
}

/// Draw the clock widget into the given rect.
pub fn draw(canvas: &Canvas, fc: &FontCollection, data: &BarData, rect: Rect) {
    draw_pill(
        canvas,
        rect,
        style::WIDGET_RADIUS,
        style::WIDGET_BG,
        style::WIDGET_BORDER,
        style::WIDGET_BORDER_WIDTH,
    );

    let date_h = text_height(
        fc,
        &data.date,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_XS,
    );
    let time_h = text_height(
        fc,
        &data.time,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_BASE,
    );
    let total_h = date_h + time_h;
    let start_y = rect.top + (rect.height() - total_h) / 2.0;

    let date_w = measure_text(
        fc,
        &data.date,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_XS,
    );
    let time_w = measure_text(
        fc,
        &data.time,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_BASE,
    );

    // Center date
    let date_x = rect.left + (rect.width() - date_w) / 2.0;
    draw_text(
        canvas,
        fc,
        &data.date,
        date_x,
        start_y,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_XS,
        style::TEXT_COLOR,
    );

    // Center time
    let time_x = rect.left + (rect.width() - time_w) / 2.0;
    draw_text(
        canvas,
        fc,
        &data.time,
        time_x,
        start_y + date_h,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_BASE,
        style::TEXT_COLOR,
    );
}
