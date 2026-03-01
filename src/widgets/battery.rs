use battery::{Manager, State as BatteryState};
use iced::widget::{button, row, text};
use iced::{Element, Length};

use crate::FONT_TEXT;

#[derive(Debug)]
pub struct State {
    manager: Option<Manager>,
    percent: Option<u8>,
    charging: bool,
}

impl Default for State {
    fn default() -> Self {
        let mut this = Self {
            manager: Manager::new().ok(),
            percent: None,
            charging: false,
        };
        this.refresh();
        this
    }
}

impl State {
    pub fn refresh(&mut self) {
        let Some(manager) = &self.manager else {
            self.percent = None;
            self.charging = false;
            return;
        };

        let Ok(mut batteries) = manager.batteries() else {
            self.percent = None;
            self.charging = false;
            return;
        };

        let Some(Ok(battery)) = batteries.next() else {
            self.percent = None;
            self.charging = false;
            return;
        };

        let pct = (battery.state_of_charge().value * 100.0)
            .clamp(0.0, 100.0)
            .round() as u8;
        self.percent = Some(pct);
        self.charging = matches!(battery.state(), BatteryState::Charging);
    }

    pub fn view<'a>(&self) -> Element<'a, crate::Message> {
        let Some(percent) = self.percent else {
            return text(String::new()).into();
        };

        let icon = if self.charging {
            match percent {
                91.. => text("\u{F0085}"),   // full
                81..91 => text("\u{F008B}"),
                71..81 => text("\u{F008A}"),
                61..71 => text("\u{F089E}"),
                51..61 => text("\u{F0089}"),
                41..51 => text("\u{F089D}"),
                31..41 => text("\u{F0088}"),
                21..31 => text("\u{F0087}"),
                11..21 => text("\u{F0086}"),
                0..11 => text("\u{F089C}"),
            }
        } else {
            match percent {
                91.. => text("\u{F0079}"),   // full
                81..91 => text("\u{F0082}"),
                71..81 => text("\u{F0081}"),
                61..71 => text("\u{F0080}"),
                51..61 => text("\u{F007F}"),
                41..51 => text("\u{F007E}"),
                31..41 => text("\u{F007D}"),
                21..31 => text("\u{F007C}"),
                11..21 => text("\u{F007B}"),
                0..11 => text("\u{F007A}"),
            }
        }
        .font(FONT_TEXT)
        .size(24)
        .color(crate::hex!(0x000000));

        let data = row![
            icon,
            text(format!("{percent}%")).color(crate::hex!(0x000000)),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center);

        button(data)
            .style(crate::widget_container_style)
            .padding([2, 6])
            .into()
    }
}
