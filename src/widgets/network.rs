use iced::widget::{button, column, row, text};
use iced::{Alignment, Element};
use iced_fonts::lucide;
use std::time::Instant;
use sysinfo::Networks;

#[derive(Debug)]
pub struct State {
    networks: Networks,
    net_last_refresh: Instant,
    net_upload: f64,
    net_download: f64,
}

impl Default for State {
    fn default() -> Self {
        let mut networks = Networks::new_with_refreshed_list();
        let (net_upload, net_download) = get_net_state(&mut networks);
        Self {
            networks: Networks::new_with_refreshed_list(),
            net_last_refresh: Instant::now(),
            net_download: net_download as f64,
            net_upload: net_upload as f64,
        }
    }
}

impl State {
    pub fn refresh(&mut self, now: Instant) {
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
        let icon = lucide::arrow_up_down()
            .size(16)
            .color(crate::hex!(0x000000));

        let upload = column![
            text("UP").size(8).color(crate::hex!(0x000000)),
            text(format_k(self.net_upload))
                .size(16)
                .color(crate::hex!(0x000000)),
        ]
        .align_x(Alignment::Center);

        let download = column![
            text("DOWN").size(8).color(crate::hex!(0x000000)),
            text(format_k(self.net_download))
                .size(16)
                .color(crate::hex!(0x000000)),
        ]
        .align_x(Alignment::Center);

        let data = row![icon, row![upload, download].spacing(4)]
            .spacing(4)
            .align_y(iced::Alignment::Center);

        button(data)
            .padding([2, 6])
            .style(crate::widget_container_style)
            .into()
    }
}

/// Format bytes/s as K: < 1 K → "0 K", < 100 K → one decimal, else integer.
fn format_k(bytes_per_sec: f64) -> String {
    if bytes_per_sec < 1024.0 {
        return " 0.0K".to_string();
    }

    let kb = bytes_per_sec / 1024.0;
    if kb < 100.0 {
        return format!("{:2.1}K", kb);
    }
    if kb < 1000.0 {
        return format!("{:3.0}K", kb);
    }
    if kb < 1024.0 {
        return " 1.0M".to_string();
    }
    return format!("{:3.0}M", kb / 1024.0);
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
