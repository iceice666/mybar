//! Linux renderer stub.
//!
//! Future: use skia-safe with vulkan or gl feature and wl_surface for Wayland.

use skia_safe::textlayout::FontCollection;
use winit::window::Window;

pub struct Renderer {
    pub font_collection: FontCollection,
    #[allow(dead_code)]
    scale_factor: f32,
}

impl Renderer {
    /// Create a new renderer. Panics (Vulkan/GL backend not implemented).
    pub fn new(_window: &Window) -> Self {
        let _ = _window;
        panic!(
            "Linux/Vulkan rendering not implemented. Future: use skia-safe with vulkan or gl \
             feature and wl_surface for Wayland."
        );
    }

    pub fn resize(&mut self, _width: u32, _height: u32) {}

    pub fn set_scale_factor(&mut self, scale: f32) {
        let _ = scale;
    }

    pub fn frame(&mut self, _draw_fn: impl FnOnce(&skia_safe::Canvas, f32, f32)) {}
}
