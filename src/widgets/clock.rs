use chrono::Local;
use iced::widget::{button, column, text};
use iced::{Element, Subscription, Task};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct State {
    date: String,
    time: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
}

impl Default for State {
    fn default() -> Self {
        let mut state = Self {
            date: String::new(),
            time: String::new(),
        };
        state.refresh();
        state
    }
}

impl State {
    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => self.refresh(),
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)
    }

    fn refresh(&mut self) {
        let now = Local::now();
        self.date = now.format("%a %b %d").to_string();
        self.time = now.format("%H:%M").to_string();
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let content = column![
            text(self.date.clone())
                .size(10)
                .color(crate::hex!(0x000000)),
            text(self.time.clone())
                .size(16)
                .color(crate::hex!(0x000000)),
        ]
        .spacing(0)
        .align_x(iced::Alignment::Center);

        button(content)
            .padding([0, 6])
            .style(crate::style::widget_container_style)
            .into()
    }
}
