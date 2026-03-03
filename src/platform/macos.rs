use super::DisplaySpec;
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSColor, NSMainMenuWindowLevel, NSScreen, NSView,
    NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

pub fn primary_display() -> DisplaySpec {
    let Some(mtm) = MainThreadMarker::new() else {
        return DisplaySpec {
            x: 0.0,
            width: 1024.0,
        };
    };

    let Some(screen) =
        NSScreen::mainScreen(mtm).or_else(|| NSScreen::screens(mtm).to_vec().into_iter().next())
    else {
        return DisplaySpec {
            x: 0.0,
            width: 1024.0,
        };
    };

    let frame = screen.frame();
    DisplaySpec {
        x: frame.origin.x as f32,
        width: frame.size.width as f32,
    }
}

pub fn configure_bar_window(window: &winit::window::Window, bar_height: f32) -> Result<(), String> {
    let mtm = MainThreadMarker::new().ok_or_else(|| String::from("must run on main thread"))?;

    let handle = window
        .window_handle()
        .map_err(|e| format!("failed to get native handle: {e}"))?;

    let ns_view: Retained<NSView> = match handle.as_raw() {
        RawWindowHandle::AppKit(app_kit) => {
            let ptr = app_kit.ns_view.as_ptr().cast::<NSView>() as *mut NSView;
            // SAFETY: Pointer comes from a live AppKit window handle.
            unsafe { Retained::retain(ptr) }
                .ok_or_else(|| String::from("failed to retain NSView"))?
        }
        _ => return Err(String::from("not running on AppKit")),
    };

    let ns_window = ns_view
        .window()
        .ok_or_else(|| String::from("NSView has no NSWindow"))?;
    let screen = ns_window
        .screen()
        .or_else(|| NSScreen::mainScreen(mtm))
        .ok_or_else(|| String::from("no NSScreen available"))?;

    let frame = screen.frame();
    let target_frame = NSRect::new(
        NSPoint::new(
            frame.origin.x,
            frame.origin.y + frame.size.height - f64::from(bar_height),
        ),
        NSSize::new(frame.size.width, f64::from(bar_height)),
    );

    // Force borderless bar style so traffic lights/titlebar do not consume top area.
    ns_window.setStyleMask(NSWindowStyleMask::Borderless);
    ns_window.setCollectionBehavior(NSWindowCollectionBehavior::CanJoinAllSpaces);
    ns_window.setLevel(NSMainMenuWindowLevel);
    ns_window.setOpaque(false);
    ns_window.setBackgroundColor(Some(&NSColor::clearColor()));
    ns_window.setFrame_display(target_frame, true);
    // SAFETY: We keep ownership in Rust and do not want AppKit autoreleasing behavior.
    unsafe { ns_window.setReleasedWhenClosed(false) };

    Ok(())
}

#[allow(dead_code)]
pub fn is_dark_mode() -> bool {
    let Some(mtm) = MainThreadMarker::new() else {
        return false;
    };

    let app = NSApplication::sharedApplication(mtm);
    let appearance = app.effectiveAppearance();
    appearance
        .name()
        .to_string()
        .to_lowercase()
        .contains("dark")
}

pub fn hide_from_dock() {
    // Cursor's seatbelt sandbox blocks the LaunchServices calls used by
    // activation policy changes, which causes noisy XPC errors in `cargo run`.
    if std::env::var_os("CURSOR_SANDBOX").is_some() {
        return;
    }

    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };

    let app = NSApplication::sharedApplication(mtm);
    let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
}
