use chrono::Local;
use iced::widget::text;
use iced::Element;

#[derive(Debug, Clone)]
pub struct State {
    value: String,
}

impl Default for State {
    fn default() -> Self {
        let mut state = Self {
            value: String::new(),
        };
        state.refresh();
        state
    }
}

impl State {
    pub fn refresh(&mut self) {
        self.value = Local::now().format("%H:%M").to_string();
    }

    pub fn view<'a>(&'a self) -> Element<'a, crate::Message> {
        text(self.value.clone()).into()
    }
}
