use iced::widget::{Space, button, container, row};
use iced::{
    Background, Border, Color, Element, Font, Length, Point, Size, Subscription, Task, Theme, time,
    window,
};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

mod platform;
mod theme;
mod widgets;

const BAR_HEIGHT: f32 = 36.0;

/// All widgets: bg #ffffff66, fg #000000
pub fn widget_container_style(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Background::Color(crate::hex!(0xffffff66)).into(),
        border: Border {
            color: crate::hex!(0x00000022),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Focused workspace badge: distinct bg for testing (#DDAACC66)
pub fn focused_widget_container_style(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Background::Color(crate::hex!(0xDDAACC66)).into(),
        border: Border {
            color: crate::hex!(0xDDAACC66),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

fn bar_container_style() -> container::Style {
    container::Style::default()
        .background(Background::Color(Color::TRANSPARENT))
        .border(Border {
            color: crate::hex!(0x00000022),
            width: 1.0,
            radius: 8.0.into(),
        })
}

/// Container style matching widget buttons (bg #ffffff66) for the apps area etc.
pub fn widget_container_style_container() -> container::Style {
    container::Style::default()
        .background(Background::Color(crate::hex!(0xffffff66)))
        .border(Border {
            color: crate::hex!(0x00000022),
            width: 1.0,
            radius: 6.0.into(),
        })
}

pub const FONT_TEXT: Font = Font {
    family: iced::font::Family::Name("Cascadia Code NF"),
    ..Font::DEFAULT
};

pub const FONT_ICON: Font = Font {
    family: iced::font::Family::Name("sketchybar-app-font"),
    ..Font::DEFAULT
};

struct BarApp {
    windows: BTreeMap<window::Id, platform::DisplaySpec>,
    dock_hidden: bool,
    aerospace: widgets::aerospace::State,
    now_playing: widgets::now_playing::State,
    perf: widgets::perf::State,
    network: widgets::network::State,
    battery: widgets::battery::State,
    clock: widgets::clock::State,
}

impl Default for BarApp {
    fn default() -> Self {
        Self {
            windows: BTreeMap::new(),
            dock_hidden: false,
            aerospace: widgets::aerospace::State::default(),
            now_playing: widgets::now_playing::State::default(),
            perf: widgets::perf::State::default(),
            network: widgets::network::State::default(),
            battery: widgets::battery::State::default(),
            clock: widgets::clock::State::default(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    WindowCreated(window::Id, platform::DisplaySpec),
    WindowEvent(window::Id, window::Event),
    FastTick,
    AerospaceFallbackTick,
    MediumTick(Instant),
    SlowTick,
    ClockTick,
    AerospaceUpdated(widgets::aerospace::Data),
    AerospaceFocusedWorkspaceUpdated(Option<String>),
    AerospaceModeUpdated(Option<String>),
    AerospaceFocusedAppsUpdated(Vec<String>),
    NowPlayingUpdated(Option<widgets::now_playing::Data>),
}

fn boot() -> (BarApp, Task<Message>) {
    let displays = platform::displays();
    let mut tasks: Vec<Task<Message>> = displays.into_iter().map(open_display_window).collect();
    tasks.push(Task::perform(
        widgets::aerospace::load_data(),
        Message::AerospaceUpdated,
    ));
    tasks.push(Task::perform(
        widgets::now_playing::load_data(),
        Message::NowPlayingUpdated,
    ));

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
        Message::FastTick => Task::batch(vec![
            Task::perform(
                widgets::aerospace::load_focused_workspace_from_bridge(),
                Message::AerospaceFocusedWorkspaceUpdated,
            ),
            Task::perform(
                widgets::aerospace::load_mode_from_bridge(),
                Message::AerospaceModeUpdated,
            ),
        ]),
        Message::AerospaceFallbackTick => Task::batch(vec![
            Task::perform(widgets::aerospace::load_data(), Message::AerospaceUpdated),
            Task::perform(
                widgets::aerospace::load_apps_for_workspace(state.aerospace.focused_workspace()),
                Message::AerospaceFocusedAppsUpdated,
            ),
        ]),
        Message::MediumTick(now) => {
            state.perf.refresh(now);
            state.network.refresh(now);
            Task::perform(
                widgets::now_playing::load_data(),
                Message::NowPlayingUpdated,
            )
        }
        Message::SlowTick => {
            state.battery.refresh();
            Task::none()
        }
        Message::ClockTick => {
            state.clock.refresh();
            Task::none()
        }
        Message::AerospaceUpdated(data) => {
            let focused = data.focused_workspace.clone();
            state.aerospace.apply(data);
            if let Some(workspace) = focused {
                Task::perform(
                    widgets::aerospace::load_apps_for_workspace(Some(workspace)),
                    Message::AerospaceFocusedAppsUpdated,
                )
            } else {
                Task::none()
            }
        }
        Message::AerospaceFocusedWorkspaceUpdated(focused_workspace) => {
            if state
                .aerospace
                .set_focused_workspace(focused_workspace.clone())
            {
                Task::perform(
                    widgets::aerospace::load_apps_for_workspace(focused_workspace),
                    Message::AerospaceFocusedAppsUpdated,
                )
            } else {
                Task::none()
            }
        }
        Message::AerospaceModeUpdated(mode) => {
            state.aerospace.set_mode(mode);
            Task::none()
        }
        Message::AerospaceFocusedAppsUpdated(apps) => {
            state.aerospace.set_apps_in_focused_workspace(apps);
            Task::none()
        }
        Message::NowPlayingUpdated(data) => {
            state.now_playing.apply(data);
            Task::none()
        }
    }
}

fn subscription(_state: &BarApp) -> Subscription<Message> {
    Subscription::batch(vec![
        window::events().map(|(id, event)| Message::WindowEvent(id, event)),
        time::every(Duration::from_millis(500)).map(|_| Message::FastTick),
        time::every(Duration::from_secs(10)).map(|_| Message::AerospaceFallbackTick),
        time::every(Duration::from_secs(2)).map(Message::MediumTick),
        time::every(Duration::from_secs(30)).map(|_| Message::SlowTick),
        time::every(Duration::from_secs(1)).map(|_| Message::ClockTick),
    ])
}

fn view<'a>(state: &'a BarApp, id: window::Id) -> Element<'a, Message> {
    if !state.windows.contains_key(&id) {
        return container(row![])
            .height(Length::Fill)
            .width(Length::Fill)
            .padding([4.0, 12.0])
            .style(|_| bar_container_style())
            .into();
    }

    let left = row![
        state.aerospace.view_mode(),
        state.aerospace.view_workspaces(),
        state.aerospace.view_apps(),
    ]
    .spacing(8)
    .height(Length::Fill)
    .align_y(iced::Alignment::Center);

    let right = row![
        state.now_playing.view(),
        state.perf.view_cpu_ram(),
        state.network.view(),
        state.battery.view(),
        state.clock.view(),
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
    .style(|_| bar_container_style())
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
        .default_font(FONT_TEXT)
        .font(iced_fonts::LUCIDE_FONT_BYTES)
        // TODO: Build a CI for updating this font automatically
        // https://github.com/kvndrsslr/sketchybar-app-font/releases
        .font(include_bytes!("../assets/sketchybar-app-font.ttf").as_slice())
        .run()
}
