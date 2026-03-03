#[cfg(not(any(target_os = "macos", target_os = "linux")))]
compile_error!("mybar only supports macOS and Linux");

use std::sync::Arc;

#[cfg(target_os = "macos")]
use objc2::rc::autoreleasepool;
use tokio::sync::watch;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition, Size};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowAttributes, WindowId, WindowLevel};

mod data;
mod layout;
mod platform;
mod render;
mod style;
mod theme;
mod widgets;

// ── User event to wake the event loop from tokio ─────────────────────────────

#[derive(Debug, Clone)]
enum UserEvent {
    DataChanged,
}

// ── Per-window state ─────────────────────────────────────────────────────────

struct BarWindow {
    window: Window,
    renderer: render::Renderer,
}

// ── Application state ────────────────────────────────────────────────────────

struct App {
    window: Option<BarWindow>,
    display: platform::DisplaySpec,
    dock_hidden: bool,
    data_rx: watch::Receiver<data::BarData>,
    _rt: tokio::runtime::Runtime,
}

impl App {
    fn new(proxy: EventLoopProxy<UserEvent>) -> Self {
        let display = platform::primary_display();

        // Build tokio runtime
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio runtime");

        // Redraw notifier: sends UserEvent::DataChanged to wake the event loop
        let notifier: data::RedrawNotifier = Arc::new(move || {
            let _ = proxy.send_event(UserEvent::DataChanged);
        });

        let data_rx = data::spawn_collectors(rt.handle(), notifier);

        Self {
            window: None,
            display,
            dock_hidden: false,
            data_rx,
            _rt: rt,
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let size = LogicalSize::new(self.display.width as f64, style::BAR_HEIGHT as f64);

        let mut attrs = WindowAttributes::default();
        attrs.inner_size = Some(Size::Logical(size.into()));
        attrs.title = "mybar".to_string();
        attrs.decorations = false;
        attrs.transparent = true;
        attrs.window_level = WindowLevel::AlwaysOnTop;
        attrs.position = Some(winit::dpi::Position::Physical(PhysicalPosition::new(
            self.display.x as i32,
            0,
        )));

        let window = event_loop
            .create_window(attrs)
            .expect("failed to create window");

        // Configure native window properties (menu bar level, join all spaces, etc.)
        let _ = platform::configure_bar_window(&window, style::BAR_HEIGHT);

        if !self.dock_hidden {
            platform::hide_from_dock();
            self.dock_hidden = true;
        }

        let renderer = render::Renderer::new(&window);
        self.window = Some(BarWindow { window, renderer });

        if let Some(bw) = &self.window {
            bw.window.request_redraw();
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::DataChanged => {
                if let Some(bw) = &self.window {
                    bw.window.request_redraw();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(bw) = self.window.as_mut() else {
            return;
        };

        if bw.window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::Resized(size) => {
                bw.renderer.resize(size.width, size.height);
                bw.window.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                bw.renderer.set_scale_factor(scale_factor as f32);
                let size = bw.window.inner_size();
                bw.renderer.resize(size.width, size.height);
                bw.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let data = self.data_rx.borrow().clone();
                let fc = bw.renderer.font_collection.clone();

                bw.renderer.frame(|canvas, w, h| {
                    // Clear to transparent
                    canvas.clear(skia_safe::Color4f::new(0.0, 0.0, 0.0, 0.0));

                    // Compute layout
                    let lo = layout::compute(&fc, &data, w, h);

                    // Draw all widgets
                    widgets::wm::draw_mode(canvas, &fc, &data, lo.mode);
                    widgets::wm::draw_workspaces_grouped(canvas, &fc, &data, lo.workspaces);
                    widgets::now_playing::draw(canvas, &fc, &data, lo.now_playing);
                    widgets::perf::draw(canvas, &fc, &data, lo.perf);
                    widgets::network::draw(canvas, &fc, &data, lo.network);
                    widgets::battery::draw(canvas, &fc, &data, lo.battery);
                    widgets::clock::draw(canvas, &fc, &data, lo.clock);
                });
            }
            WindowEvent::CloseRequested => {
                // Bar should not be closeable, but handle gracefully
            }
            _ => {}
        }
    }
}

// ── Entry point ──────────────────────────────────────────────────────────────

fn main() {
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("failed to create event loop");

    let proxy = event_loop.create_proxy();
    let mut app = App::new(proxy);

    #[cfg(target_os = "macos")]
    autoreleasepool(|_| {
        event_loop.run_app(&mut app).expect("event loop failed");
    });

    #[cfg(not(target_os = "macos"))]
    event_loop.run_app(&mut app).expect("event loop failed");
}
