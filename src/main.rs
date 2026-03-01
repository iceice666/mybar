use iced::widget::{container, text};
use iced::{window, Element, Length, Point, Size, Subscription, Task};
use std::collections::BTreeMap;

mod platform;

const BAR_HEIGHT: f32 = 32.0;

#[derive(Debug, Default)]
struct BarApp {
    windows: BTreeMap<window::Id, platform::DisplaySpec>,
}

#[derive(Debug, Clone)]
enum Message {
    WindowCreated(window::Id, platform::DisplaySpec),
    WindowEvent(window::Id, window::Event),
}

fn boot() -> (BarApp, Task<Message>) {
    let displays = platform::displays();
    let tasks = displays.into_iter().map(open_display_window);

    (BarApp::default(), Task::batch(tasks))
}

fn update(state: &mut BarApp, message: Message) -> Task<Message> {
    match message {
        Message::WindowCreated(id, display) => {
            state.windows.insert(id, display);
            reconcile_window(id)
        }
        Message::WindowEvent(id, event) => {
            if !state.windows.contains_key(&id) {
                return Task::none();
            }

            match event {
                window::Event::Opened { .. } | window::Event::Resized(_) => {
                    reconcile_window(id)
                }
                _ => Task::none(),
            }
        }
    }
}

fn subscription(_state: &BarApp) -> Subscription<Message> {
    window::events().map(|(id, event)| Message::WindowEvent(id, event))
}

fn view<'a>(state: &'a BarApp, id: window::Id) -> Element<'a, Message> {
    let label = match state.windows.get(&id) {
        Some(display) => format!("mybar test | display {}", display.index + 1),
        None => String::from("mybar test | initializing"),
    };

    container(text(label))
        .height(Length::Fill)
        .width(Length::Fill)
        .padding([0, 12])
        .into()
}

fn title(_state: &BarApp, _id: window::Id) -> String {
    String::from("mybar")
}

fn open_display_window(display: platform::DisplaySpec) -> Task<Message> {
    let settings = window::Settings {
        size: Size::new(display.width, BAR_HEIGHT),
        position: window::Position::Specific(Point::new(display.x, 0.0)),
        min_size: Some(Size::new(display.width, BAR_HEIGHT)),
        max_size: Some(Size::new(display.width, BAR_HEIGHT)),
        resizable: false,
        decorations: false,
        level: window::Level::AlwaysOnTop,
        ..window::Settings::default()
    };

    let (_, open_task) = window::open(settings);

    open_task.map(move |id| Message::WindowCreated(id, display.clone()))
}

fn reconcile_window(id: window::Id) -> Task<Message> {
    window::run(id, move |window| {
        let _ = platform::configure_bar_window(window, BAR_HEIGHT);
    })
    .discard()
}

fn main() -> iced::Result {
    iced::daemon(boot, update, view)
        .subscription(subscription)
        .title(title)
        .run()
}
