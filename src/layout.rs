//! Layout engine: computes widget rects for a horizontal bar.

use skia_safe::Rect;
use skia_safe::textlayout::FontCollection;

use crate::data::BarData;
use crate::style;
use crate::widgets;

/// All the rects needed to draw the bar.
pub struct BarLayout {
    pub mode: Rect,
    pub workspaces: Rect,
    pub now_playing: Rect,
    pub perf: Rect,
    pub network: Rect,
    pub battery: Rect,
    pub clock: Rect,
}

/// Compute layout for all widgets given the bar dimensions.
pub fn compute(fc: &FontCollection, data: &BarData, bar_width: f32, bar_height: f32) -> BarLayout {
    let inner_top = style::BAR_PADDING_V;
    let inner_height = bar_height - style::BAR_PADDING_V * 2.0;

    // ── Left section ──────────────────────────────────────────────────────────
    let mut lx = style::BAR_PADDING_H;

    let mode_w = widgets::wm::measure_mode(fc, data);
    let mode = Rect::from_xywh(lx, inner_top, mode_w, inner_height);
    lx += mode_w + style::SECTION_SPACING;

    let ws_w = widgets::wm::measure_workspaces_grouped(fc, data);
    let workspaces = Rect::from_xywh(lx, inner_top, ws_w, inner_height);

    // ── Right section (laid out right-to-left) ────────────────────────────────
    let mut rx = bar_width - style::BAR_PADDING_H;

    let clock_w = widgets::clock::measure(fc, data);
    rx -= clock_w;
    let clock = Rect::from_xywh(rx, inner_top, clock_w, inner_height);
    rx -= style::WIDGET_SPACING;

    let bat_w = widgets::battery::measure(fc, data);
    rx -= bat_w;
    let battery = Rect::from_xywh(rx, inner_top, bat_w, inner_height);
    if bat_w > 0.0 {
        rx -= style::WIDGET_SPACING;
    }

    let net_w = widgets::network::measure(fc, data);
    rx -= net_w;
    let network = Rect::from_xywh(rx, inner_top, net_w, inner_height);
    rx -= style::WIDGET_SPACING;

    let perf_w = widgets::perf::measure(fc, data);
    rx -= perf_w;
    let perf = Rect::from_xywh(rx, inner_top, perf_w, inner_height);
    rx -= style::WIDGET_SPACING;

    let np_w = widgets::now_playing::measure(fc, data);
    rx -= np_w;
    let now_playing = Rect::from_xywh(rx, inner_top, np_w, inner_height);

    BarLayout {
        mode,
        workspaces,
        now_playing,
        perf,
        network,
        battery,
        clock,
    }
}
