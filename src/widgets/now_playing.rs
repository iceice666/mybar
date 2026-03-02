//! Now Playing widget: track/artist text (no pill, just text like original).

use skia_safe::textlayout::FontCollection;
use skia_safe::{Canvas, Rect};

use crate::data::BarData;
use crate::render::{draw_text, measure_text, text_height};
use crate::style;

fn label(data: &BarData) -> Option<String> {
    let np = data.now_playing.as_ref()?;
    if np.artist.is_empty() {
        Some(np.title.clone())
    } else {
        Some(format!("{} - {}", np.title, np.artist))
    }
}

pub fn measure(fc: &FontCollection, data: &BarData) -> f32 {
    let Some(text) = label(data) else {
        return 0.0;
    };
    measure_text(fc, &text, style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM)
}

pub fn draw(canvas: &Canvas, fc: &FontCollection, data: &BarData, rect: Rect) {
    let Some(text) = label(data) else {
        return;
    };

    let h = text_height(fc, &text, style::FONT_FAMILY_TEXT, style::FONT_SIZE_SM);
    let y = rect.top + (rect.height() - h) / 2.0;

    draw_text(
        canvas,
        fc,
        &text,
        rect.left,
        y,
        style::FONT_FAMILY_TEXT,
        style::FONT_SIZE_SM,
        style::TEXT_COLOR,
    );
}
