use iced::widget::{column, row, text};
use iced::Element;
use std::time::Instant;
use sysinfo::{Networks, System};

#[derive(Debug)]
pub struct State {
    system: System,
    networks: Networks,
    cpu_percent: f32,
    mem_percent: f32,
    net_up_per_sec: u64,
    net_down_per_sec: u64,
    last_sample: Option<(Instant, u64, u64)>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            system: System::new_all(),
            networks: Networks::new_with_refreshed_list(),
            cpu_percent: 0.0,
            mem_percent: 0.0,
            net_up_per_sec: 0,
            net_down_per_sec: 0,
            last_sample: None,
        }
    }
}

impl State {
    pub fn refresh(&mut self, now: Instant) {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();

        self.cpu_percent = self.system.global_cpu_usage();
        let total_mem = self.system.total_memory();
        self.mem_percent = if total_mem == 0 {
            0.0
        } else {
            (self.system.used_memory() as f32 / total_mem as f32) * 100.0
        };

        self.networks.refresh(true);
        let (total_received, total_transmitted) = self
            .networks
            .iter()
            .fold((0_u64, 0_u64), |(received, transmitted), (_, network)| {
                (
                    received.saturating_add(network.total_received()),
                    transmitted.saturating_add(network.total_transmitted()),
                )
            });

        if let Some((last_when, last_rx, last_tx)) = self.last_sample {
            let elapsed = now.saturating_duration_since(last_when).as_secs_f64();
            if elapsed > 0.0 {
                self.net_down_per_sec = ((total_received.saturating_sub(last_rx)) as f64 / elapsed)
                    .round() as u64;
                self.net_up_per_sec = ((total_transmitted.saturating_sub(last_tx)) as f64 / elapsed)
                    .round() as u64;
            }
        }
        self.last_sample = Some((now, total_received, total_transmitted));
    }

    pub fn view<'a>(&self) -> Element<'a, crate::Message> {
        let net = column![
            text(format!("U {}", format_rate(self.net_up_per_sec))),
            text(format!("D {}", format_rate(self.net_down_per_sec))),
        ]
        .spacing(0);

        row![
            text(format!("PERF | CPU {:>3.0}% MEM {:>3.0}%", self.cpu_percent, self.mem_percent)),
            net
        ]
        .spacing(8)
        .into()
    }
}

fn format_rate(bytes_per_second: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;

    let bps = bytes_per_second as f64;
    if bps >= MB {
        format!("{:.1} MB/s", bps / MB)
    } else if bps >= KB {
        format!("{:.1} KB/s", bps / KB)
    } else {
        format!("{bytes_per_second} B/s")
    }
}
