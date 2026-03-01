use iced::widget::{button, column, row, text};
use iced::{Alignment, Element, Subscription, Task};
use iced_fonts::lucide;
use std::time::{Duration, Instant};
use sysinfo::System;

#[derive(Debug)]
pub struct State {
    system: System,
    cpu_percent: f32,
    mem_used: u64,
    mem_total: u64,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick(Instant),
}

impl Default for State {
    fn default() -> Self {
        let system = System::new_all();
        let mem_total = system.total_memory();
        Self {
            system,
            cpu_percent: 0.0,
            mem_used: 0,
            mem_total,
        }
    }
}

impl State {
    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(now) => self.refresh(now),
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_secs(2)).map(Message::Tick)
    }

    fn refresh(&mut self, _now: Instant) {
        self.system.refresh_cpu_all();
        self.cpu_percent = self.system.global_cpu_usage();

        self.system.refresh_memory();
        self.mem_used = self.system.used_memory();
    }

    pub fn view_cpu_ram<'a>(&self) -> Element<'a, Message> {
        let format_mem = |bytes: u64| {
            let (value, unit) = format_number(bytes as f64);
            format!("{:>2.1}{}", value, unit)
        };

        let cpu = column![
            text("CPU").size(8).color(crate::hex!(0x000000)),
            text(format!("{:>3.0}%", self.cpu_percent)).color(crate::hex!(0x000000)),
        ]
        .align_x(Alignment::Center);
        let mem = column![
            text(format!("MEM /{}", format_mem(self.mem_total)))
                .size(8)
                .color(crate::hex!(0x000000)),
            text(format_mem(self.mem_used)).color(crate::hex!(0x000000)),
        ]
        .align_x(Alignment::Center);

        let data = row![
            lucide::cpu().size(14).color(crate::hex!(0x000000)),
            row![cpu, mem].spacing(4),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center);

        button(data)
            .padding([2, 6])
            .style(crate::style::widget_container_style)
            .into()
    }
}

fn format_number(bytes: f64) -> (f64, char) {
    let mut value = bytes.max(0.0);
    let mut unit = ' ';

    for candidate in ['K', 'M', 'G', 'T'] {
        if value < 1024.0 {
            break;
        }
        value /= 1024.0;
        unit = candidate;
    }

    (value, unit)
}
