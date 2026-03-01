use battery::{Manager, State as BatteryState};
use iced::Element;
use iced::widget::{row, text};

#[derive(Debug)]
pub struct State {
    manager: Option<Manager>,
    percent: Option<u8>,
    charging: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            manager: Manager::new().ok(),
            percent: None,
            charging: false,
        }
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

        let icon = match percent {
            100.. => text("\u{F0079}"),
            75..100 => text("\u{F007A}"),
            50..75 => text("\u{F007B}"),
            25..50 => text("\u{F007C}"),
            0..25 => text("\u{F007D}"),
        }
        .size(14);

        row![icon, text(format!("{percent}%"))]
            .spacing(4)
            .align_y(iced::Alignment::Center)
            .into()
    }
}
