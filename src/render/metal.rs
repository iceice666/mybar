//! macOS Metal renderer backend.

use objc2::rc::{autoreleasepool, Retained};
use objc2::runtime::ProtocolObject;
use objc2_core_foundation::CGSize;
use objc2_metal::{MTLCommandBuffer, MTLCommandQueue, MTLCreateSystemDefaultDevice, MTLDevice};
use objc2_quartz_core::CAMetalDrawable;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use skia_safe::gpu::{self, backend_render_targets, mtl, DirectContext, SurfaceOrigin};
use skia_safe::textlayout::FontCollection;
use skia_safe::{Canvas, ColorType};
use winit::window::Window;

use objc2_app_kit::NSView;
use objc2_quartz_core::CAMetalLayer;

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

            let view_ptr = match window.window_handle().unwrap().as_raw() {
                RawWindowHandle::AppKit(appkit) => appkit.ns_view.as_ptr() as *mut NSView,
                _ => panic!("Unsupported window handle"),
            };

            let view = unsafe { view_ptr.as_ref().unwrap() };
            view.setWantsLayer(true);
            view.setLayer(Some(&layer.clone().into_super()));

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
        let font_collection = super::build_font_collection();

        Self {
            metal_layer,
            command_queue,
            skia,
            font_collection,
            scale_factor,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.metal_layer
            .setDrawableSize(CGSize::new(width as f64, height as f64));
    }

    pub fn set_scale_factor(&mut self, scale: f32) {
        self.scale_factor = scale;
        self.metal_layer.setContentsScale(scale as f64);
    }

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
