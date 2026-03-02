//! Skia Metal renderer: manages GPU context, surfaces, and drawing helpers.

use objc2::rc::{autoreleasepool, Retained};
use objc2::runtime::ProtocolObject;
use objc2_core_foundation::CGSize;
use objc2_metal::{MTLCommandBuffer, MTLCommandQueue, MTLCreateSystemDefaultDevice, MTLDevice};
use objc2_quartz_core::CAMetalDrawable;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use skia_safe::gpu::{self, backend_render_targets, mtl, DirectContext, SurfaceOrigin};
use skia_safe::textlayout::{
    FontCollection, ParagraphBuilder, ParagraphStyle, TextStyle, TypefaceFontProvider,
};
use skia_safe::{Canvas, Color4f, ColorType, FontMgr, Paint, PaintStyle, RRect, Rect};
use winit::window::Window;

#[cfg(target_os = "macos")]
use objc2_app_kit::NSView;
use objc2_quartz_core::CAMetalLayer;

use crate::style;

// ── Per-window renderer ──────────────────────────────────────────────────────

pub struct Renderer {
    pub metal_layer: Retained<CAMetalLayer>,
    pub command_queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    pub skia: DirectContext,
    pub font_collection: FontCollection,
    pub scale_factor: f32,
}

impl Renderer {
    /// Create a new renderer attached to the given winit window.
    pub fn new(window: &Window) -> Self {
        let device = MTLCreateSystemDefaultDevice().expect("no Metal device found");
        let scale_factor = window.scale_factor() as f32;

        let metal_layer = {
            let layer = CAMetalLayer::new();
            layer.setDevice(Some(&device));
            layer.setPixelFormat(objc2_metal::MTLPixelFormat::BGRA8Unorm);
            layer.setPresentsWithTransaction(false);
            layer.setFramebufferOnly(false);
            layer.setOpaque(false);
            layer.setContentsScale(scale_factor as f64);

            let size = window.inner_size();
            layer.setDrawableSize(CGSize::new(size.width as f64, size.height as f64));

            // Attach layer to NSView
            let view_ptr = match window.window_handle().unwrap().as_raw() {
                #[cfg(target_os = "macos")]
                RawWindowHandle::AppKit(appkit) => appkit.ns_view.as_ptr() as *mut NSView,
                _ => panic!("Unsupported window handle"),
            };

            #[cfg(target_os = "macos")]
            {
                let view = unsafe { view_ptr.as_ref().unwrap() };
                view.setWantsLayer(true);
                view.setLayer(Some(&layer.clone().into_super()));
            }

            layer
        };

        let command_queue = device
            .newCommandQueue()
            .expect("unable to get command queue");

        let backend = unsafe {
            mtl::BackendContext::new(
                Retained::as_ptr(&device) as mtl::Handle,
                Retained::as_ptr(&command_queue) as mtl::Handle,
            )
        };

        let skia = gpu::direct_contexts::make_metal(&backend, None).unwrap();

        // Build font collection with custom fonts
        let font_collection = build_font_collection();

        Self {
            metal_layer,
            command_queue,
            skia,
            font_collection,
            scale_factor,
        }
    }

    /// Resize the Metal drawable to match the new window size.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.metal_layer
            .setDrawableSize(CGSize::new(width as f64, height as f64));
    }

    /// Update the scale factor (e.g. when moving between displays).
    pub fn set_scale_factor(&mut self, scale: f32) {
        self.scale_factor = scale;
        self.metal_layer.setContentsScale(scale as f64);
    }

    /// Render a frame. Calls `draw_fn` with the Skia canvas and **logical** size
    /// (i.e. physical pixels / scale_factor). The canvas has a scale transform
    /// applied so drawing in logical coordinates maps to physical pixels.
    pub fn frame(&mut self, draw_fn: impl FnOnce(&Canvas, f32, f32)) {
        autoreleasepool(|_| {
            let Some(drawable) = self.metal_layer.nextDrawable() else {
                return;
            };

            let size = self.metal_layer.drawableSize();
            let (pw, ph) = (size.width as f32, size.height as f32);
            let scale = self.scale_factor;
            let (lw, lh) = (pw / scale, ph / scale);

            let mut surface = {
                let texture_info = unsafe {
                    mtl::TextureInfo::new(Retained::as_ptr(&drawable.texture()) as mtl::Handle)
                };
                let backend_rt =
                    backend_render_targets::make_mtl((pw as i32, ph as i32), &texture_info);
                gpu::surfaces::wrap_backend_render_target(
                    &mut self.skia,
                    &backend_rt,
                    SurfaceOrigin::TopLeft,
                    ColorType::BGRA8888,
                    None,
                    None,
                )
                .unwrap()
            };

            let canvas = surface.canvas();
            canvas.save();
            canvas.scale((scale, scale));
            draw_fn(canvas, lw, lh);
            canvas.restore();

            self.skia.flush_and_submit();
            drop(surface);

            let cmd = self
                .command_queue
                .commandBuffer()
                .expect("unable to get command buffer");

            let presentable: Retained<ProtocolObject<dyn objc2_metal::MTLDrawable>> =
                (&drawable).into();
            cmd.presentDrawable(&presentable);
            cmd.commit();
        });
    }
}

// ── Font loading ─────────────────────────────────────────────────────────────

fn build_font_collection() -> FontCollection {
    let mut fc = FontCollection::new();

    // System font manager (provides Cascadia Code NF etc.)
    fc.set_default_font_manager(FontMgr::default(), None);

    // Custom font provider for bundled fonts
    let mut provider = TypefaceFontProvider::new();

    // Load the sketchybar-app-font
    let app_font_data = include_bytes!("../assets/sketchybar-app-font.ttf");
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
