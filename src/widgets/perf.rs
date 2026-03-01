use iced::Element;
use iced::widget::{column, row, text};
use iced_fonts::lucide;
use std::time::Instant;
use sysinfo::{Networks, System};

#[derive(Debug)]
pub struct State {
    system: System,
    networks: Networks,
    cpu_percent: f32,
    mem_used: u64,
    mem_total: u64,
    net_last_refresh: Instant,
    net_upload: f64,
    net_download: f64,
}

impl Default for State {
    fn default() -> Self {
        let system = System::new_all();
        let mem_total = system.total_memory();
        let mut networks = Networks::new_with_refreshed_list();
        let (net_upload, net_download) = get_net_state(&mut networks);
        Self {
            system,
            networks: Networks::new_with_refreshed_list(),
            cpu_percent: 0.0,
            mem_used: 0,
            mem_total,
            net_last_refresh: Instant::now(),
            net_download: net_download as f64,
            net_upload: net_upload as f64,
        }
    }
}

impl State {
    pub fn refresh(&mut self, now: Instant) {
        self.system.refresh_cpu_all();
        self.cpu_percent = self.system.global_cpu_usage();

        self.system.refresh_memory();
        self.mem_used = self.system.used_memory();

        self.networks.refresh(true);
        let elapsed = (now - self.net_last_refresh)
            .as_secs_f64()
            .max(f64::EPSILON);
        let (last_rx, last_tx) = get_net_state(&mut self.networks);
        self.net_last_refresh = now;

        self.net_download = ewma(self.net_download, last_rx as f64 / elapsed, elapsed);
        self.net_upload = ewma(self.net_upload, last_tx as f64 / elapsed, elapsed);
    }

    pub fn view<'a>(&self) -> Element<'a, crate::Message> {
        let format_net = |bytes: f64| {
            let (value, unit) = format_number(bytes);
            format!("{:>6.1}{}B/s", value, unit)
        };

        let net = column![
            row![
                lucide::arrow_big_up().size(11),
                text(format_net(self.net_upload)).size(11),
            ]
            .spacing(3),
            row![
                lucide::arrow_big_down().size(11),
                text(format_net(self.net_download)).size(11),
            ]
            .spacing(3),
        ]
        .spacing(0);

        let format_mem = |bytes: u64| {
            let (value, unit) = format_number(bytes as f64);
            format!("{:>2.1}{}B", value, unit)
        };

        let perf = row![
            lucide::cpu().size(14),
            text(format!("{:>3.0}%", self.cpu_percent)),
            lucide::memory_stick().size(14),
            text(format!(
                "{}/{}",
                format_mem(self.mem_used),
                format_mem(self.mem_total)
            )),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center);

        row![perf, net]
            .spacing(8)
            .align_y(iced::Alignment::Center)
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

fn get_net_state(networks: &mut Networks) -> (u64, u64) {
    networks.iter().fold((0_u64, 0_u64), |(rx, tx), (_, data)| {
        let rx = rx.saturating_add(data.received());
        let tx = tx.saturating_add(data.transmitted());

        (rx, tx)
    })
}

const NET_EWMA_TAU_SECS: f64 = 4.0;
fn ewma(history: f64, sample: f64, elapsed: f64) -> f64 {
    let history_weight = (-elapsed / NET_EWMA_TAU_SECS).exp();
    sample + (history - sample) * history_weight
}
