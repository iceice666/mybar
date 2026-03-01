//! Window manager abstraction for the bar (Aerospace on macOS, pluggable on Linux).

use iced::{Element, Subscription, Task};
use std::fmt::Debug;

pub mod aerospace;
pub mod noop;

pub trait Wm: Sized + Debug {
    type Message: Debug + Clone + Send + 'static;

    fn new() -> (Self, Task<Self::Message>);
    fn update(&mut self, message: Self::Message) -> Task<Self::Message>;
    fn subscription(&self) -> Subscription<Self::Message>;
    fn view_mode(&self) -> Element<'_, Self::Message>;
    fn view_workspaces(&self) -> Element<'_, Self::Message>;
}

#[cfg(target_os = "macos")]
pub type ActiveWm = aerospace::AerospaceWm;

#[cfg(not(target_os = "macos"))]
pub type ActiveWm = noop::NoopWm;
