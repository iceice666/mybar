//! Skia renderer: manages GPU context, surfaces, and drawing helpers.
//!
//! On macOS: Metal backend. On Linux: stub (Vulkan not yet implemented).

use skia_safe::textlayout::{
    FontCollection, ParagraphBuilder, ParagraphStyle, TextStyle, TypefaceFontProvider,
};
use skia_safe::{Canvas, Color4f, FontMgr, Paint, PaintStyle, RRect, Rect};

use crate::style;

#[cfg(target_os = "macos")]
mod metal;
#[cfg(target_os = "macos")]
pub use metal::Renderer;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::Renderer;

// ── Font loading ─────────────────────────────────────────────────────────────

pub(crate) fn build_font_collection() -> FontCollection {
    let mut fc = FontCollection::new();

    // System font manager (provides Cascadia Code NF etc.)
    fc.set_default_font_manager(FontMgr::default(), None);

    // Custom font provider for bundled fonts
    let mut provider = TypefaceFontProvider::new();

    // Load the sketchybar-app-font
    let app_font_data = include_bytes!("../../assets/sketchybar-app-font.ttf");
    if let Some(tf) = FontMgr::default().new_from_data(app_font_data, None) {
        provider.register_typeface(tf, Some(style::FONT_FAMILY_ICON));
    }

    fc.set_asset_font_manager(Some(provider.into()));
    fc
}

// ── Drawing helpers ──────────────────────────────────────────────────────────

/// Draw a rounded-rect pill with optional fill and border.
pub fn draw_pill(
    canvas: &Canvas,
    rect: Rect,
    radius: f32,
    fill: Color4f,
    border: Color4f,
    border_width: f32,
) {
    let rrect = RRect::new_rect_xy(rect, radius, radius);

    // Fill
    let mut paint = Paint::new(fill, None);
    paint.set_anti_alias(true);
    canvas.draw_rrect(rrect, &paint);

    // Border
    if border_width > 0.0 {
        let mut border_paint = Paint::new(border, None);
        border_paint.set_anti_alias(true);
        border_paint.set_style(PaintStyle::Stroke);
        border_paint.set_stroke_width(border_width);
        canvas.draw_rrect(rrect, &border_paint);
    }
}

/// Measure text width using the paragraph builder (handles font shaping for ligatures).
pub fn measure_text(fc: &FontCollection, text: &str, font_family: &str, font_size: f32) -> f32 {
    let mut style = ParagraphStyle::new();
    let mut ts = TextStyle::new();
    ts.set_font_size(font_size);
    ts.set_font_families(&[font_family]);
    style.set_text_style(&ts);

    let mut builder = ParagraphBuilder::new(&style, fc);
    builder.add_text(text);
    let mut paragraph = builder.build();
    paragraph.layout(f32::MAX);
    paragraph.max_intrinsic_width()
}

/// Draw text at the given position using the paragraph system.
pub fn draw_text(
    canvas: &Canvas,
    fc: &FontCollection,
    text: &str,
    x: f32,
    y: f32,
    font_family: &str,
    font_size: f32,
    color: Color4f,
) {
    let mut style = ParagraphStyle::new();
    let mut ts = TextStyle::new();
    ts.set_font_size(font_size);
    ts.set_font_families(&[font_family]);
    ts.set_color(color.to_color());
    style.set_text_style(&ts);

    let mut builder = ParagraphBuilder::new(&style, fc);
    builder.add_text(text);
    let mut paragraph = builder.build();
    paragraph.layout(f32::MAX);
    paragraph.paint(canvas, (x, y));
}

/// Measure and get the height of text.
pub fn text_height(fc: &FontCollection, text: &str, font_family: &str, font_size: f32) -> f32 {
    let mut style = ParagraphStyle::new();
    let mut ts = TextStyle::new();
    ts.set_font_size(font_size);
    ts.set_font_families(&[font_family]);
    style.set_text_style(&ts);

    let mut builder = ParagraphBuilder::new(&style, fc);
    builder.add_text(text);
    let mut paragraph = builder.build();
    paragraph.layout(f32::MAX);
    paragraph.height()
}
