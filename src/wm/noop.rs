use super::Wm;
use iced::widget::text;
use iced::{Element, Subscription, Task};

#[cfg_attr(target_os = "macos", allow(dead_code))]
#[derive(Debug, Default)]
pub struct NoopWm;

#[cfg_attr(target_os = "macos", allow(dead_code))]
#[derive(Debug, Clone)]
pub enum Message {}

impl Wm for NoopWm {
    type Message = Message;

    fn new() -> (Self, Task<Self::Message>) {
        (Self::default(), Task::none())
    }

    fn update(&mut self, _message: Self::Message) -> Task<Self::Message> {
        Task::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn view_mode(&self) -> Element<'_, Self::Message> {
        text("--").into()
    }

    fn view_workspaces(&self) -> Element<'_, Self::Message> {
        text("--").into()
    }
}
