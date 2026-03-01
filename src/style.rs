use iced::widget::{button, container};
use iced::{Background, Border, Color, Font, Theme};

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

pub fn bar_container_style() -> container::Style {
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
