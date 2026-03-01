use iced::widget::text;
use iced::{Element, Subscription, Task};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Data {
    pub title: String,
    pub artist: String,
}

#[derive(Debug, Clone, Default)]
pub struct State {
    current: Option<Data>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Updated(Option<Data>),
}

impl State {
    pub fn new() -> (Self, Task<Message>) {
        let state = Self::default();
        let task = Task::perform(load_data(), Message::Updated);
        (state, task)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => Task::perform(load_data(), Message::Updated),
            Message::Updated(data) => {
                self.current = data;
                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_secs(2)).map(|_| Message::Tick)
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let Some(current) = &self.current else {
            return text(String::new()).into();
        };

        let label = if current.artist.is_empty() {
            current.title.clone()
        } else {
            format!("{} - {}", current.title, current.artist)
        };
        text(label).color(crate::hex!(0x000000)).into()
    }
}

#[cfg(target_os = "macos")]
pub async fn load_data() -> Option<Data> {
    use std::process::Command;

    let output = Command::new("nowplaying-cli")
        .args(["get", "title", "artist"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let content = String::from_utf8(output.stdout).ok()?;
    let mut lines = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned);
    let title = lines.next().unwrap_or_default();
    let artist = lines.next().unwrap_or_default();

    if title.is_empty() && artist.is_empty() {
        None
    } else {
        Some(Data { title, artist })
    }
}

#[cfg(not(target_os = "macos"))]
pub async fn load_data() -> Option<Data> {
    None
}
