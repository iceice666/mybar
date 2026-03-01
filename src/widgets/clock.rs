use chrono::Local;
use iced::Element;
use iced::widget::{button, column, row, text};
use iced_fonts::lucide;

#[derive(Debug, Clone)]
pub struct State {
    date: String,
    time: String,
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
    pub fn refresh(&mut self) {
        let now = Local::now();
        self.date = now.format("%a %b %d").to_string();
        self.time = now.format("%H:%M").to_string();
    }

    pub fn view<'a>(&'a self) -> Element<'a, crate::Message> {
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

        let data = row![
            lucide::calendar_clock()
                .size(16)
                .color(crate::hex!(0x000000)),
            content,
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center);

        button(data)
            .padding([0, 6])
            .style(crate::widget_container_style)
            .into()
    }
}
