use iced::widget::{Space, container, row};
use iced::{Color, Element, Length, Point, Size, Subscription, Task, window};
use std::collections::BTreeMap;

mod platform;
use crate::wm::Wm;
mod style;
mod theme;
mod widgets;
mod wm;

const BAR_HEIGHT: f32 = 36.0;

type WmMessage = <wm::ActiveWm as wm::Wm>::Message;

struct BarApp {
    windows: BTreeMap<window::Id, platform::DisplaySpec>,
    dock_hidden: bool,
    wm: wm::ActiveWm,
    now_playing: widgets::now_playing::State,
    perf: widgets::perf::State,
    network: widgets::network::State,
    battery: widgets::battery::State,
    clock: widgets::clock::State,
}

#[derive(Debug, Clone)]
enum Message {
    WindowCreated(window::Id, platform::DisplaySpec),
    WindowEvent(window::Id, window::Event),
    Wm(WmMessage),
    NowPlaying(widgets::now_playing::Message),
    Perf(widgets::perf::Message),
    Network(widgets::network::Message),
    Battery(widgets::battery::Message),
    Clock(widgets::clock::Message),
}

fn boot() -> (BarApp, Task<Message>) {
    let displays = platform::displays();
    let (wm, wm_task) = wm::ActiveWm::new();
    let (now_playing, now_playing_task) = widgets::now_playing::State::new();
    let (perf, _) = widgets::perf::State::new();
    let (network, _) = widgets::network::State::new();
    let (battery, _) = widgets::battery::State::new();
    let (clock, _) = widgets::clock::State::new();

    let mut tasks: Vec<Task<Message>> = displays.into_iter().map(open_display_window).collect();
    tasks.push(wm_task.map(Message::Wm));
    tasks.push(now_playing_task.map(Message::NowPlaying));

    let state = BarApp {
        windows: BTreeMap::new(),
        dock_hidden: false,
        wm,
        now_playing,
        perf,
        network,
        battery,
        clock,
    };

    (state, Task::batch(tasks))
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
                window::Event::Opened { .. } => {
                    if !state.dock_hidden {
                        platform::hide_from_dock();
                        state.dock_hidden = true;
                    }
                    reconcile_window(id)
                }
                window::Event::Resized(_) => reconcile_window(id),
                _ => Task::none(),
            }
        }
        Message::Wm(m) => state.wm.update(m).map(Message::Wm),
        Message::NowPlaying(m) => state.now_playing.update(m).map(Message::NowPlaying),
        Message::Perf(m) => state.perf.update(m).map(Message::Perf),
        Message::Network(m) => state.network.update(m).map(Message::Network),
        Message::Battery(m) => state.battery.update(m).map(Message::Battery),
        Message::Clock(m) => state.clock.update(m).map(Message::Clock),
    }
}

fn subscription(state: &BarApp) -> Subscription<Message> {
    Subscription::batch(vec![
        window::events().map(|(id, event)| Message::WindowEvent(id, event)),
        state.wm.subscription().map(Message::Wm),
        state.now_playing.subscription().map(Message::NowPlaying),
        state.perf.subscription().map(Message::Perf),
        state.network.subscription().map(Message::Network),
        state.battery.subscription().map(Message::Battery),
        state.clock.subscription().map(Message::Clock),
    ])
}

fn view<'a>(state: &'a BarApp, id: window::Id) -> Element<'a, Message> {
    if !state.windows.contains_key(&id) {
        return container(row![])
            .height(Length::Fill)
            .width(Length::Fill)
            .padding([4.0, 12.0])
            .style(|_| style::bar_container_style())
            .into();
    }

    let left = row![
        state.wm.view_mode().map(Message::Wm),
        state.wm.view_workspaces().map(Message::Wm),
    ]
    .spacing(8)
    .height(Length::Fill)
    .align_y(iced::Alignment::Center);

    let right = row![
        state.now_playing.view().map(Message::NowPlaying),
        state.perf.view_cpu_ram().map(Message::Perf),
        state.network.view().map(Message::Network),
        state.battery.view().map(Message::Battery),
        state.clock.view().map(Message::Clock),
    ]
    .spacing(4)
    .height(Length::Fill)
    .align_y(iced::Alignment::Center);

    container(
        row![left, Space::new().width(Length::Fill), right]
            .height(Length::Fill)
            .align_y(iced::Alignment::Center),
    )
    .align_y(iced::Alignment::Center)
    .height(Length::Fill)
    .width(Length::Fill)
    .padding([2.0, 10.0])
    .style(|_| style::bar_container_style())
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
        transparent: true,
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
        .theme(|_state: &BarApp, _window| iced::Theme::Light)
        .style(|_state: &BarApp, _theme| iced::theme::Style {
            background_color: Color::TRANSPARENT,
            text_color: crate::hex!(0x000000),
        })
        .default_font(style::FONT_TEXT)
        .font(iced_fonts::LUCIDE_FONT_BYTES)
        // TODO: Build a CI for updating this font automatically
        // https://github.com/kvndrsslr/sketchybar-app-font/releases
        .font(include_bytes!("../assets/sketchybar-app-font.ttf").as_slice())
        .run()
}
