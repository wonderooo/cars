use crate::Task;
use async_trait::async_trait;
use common::config::CONFIG;
use common::io::copart::CopartCmd;
use common::kafka::{KafkaSender, ToTopic};
use std::collections::HashMap;
use tracing::{debug, error, info};

pub struct CopartLotSearchTask {
    cmd_sender: KafkaSender,
}

impl CopartLotSearchTask {
    pub fn new(cmd_sender: KafkaSender) -> Self {
        Self { cmd_sender }
    }
}

impl Default for CopartLotSearchTask {
    fn default() -> Self {
        Self::new(KafkaSender::new(CONFIG.kafka.url.to_owned()))
    }
}

#[async_trait]
impl Task for CopartLotSearchTask {
    async fn run(&self, _opts: Option<&HashMap<String, String>>) {
        let now = chrono::Utc::now();
        let hours_in_month = 24 * 31;

        for next_hour in 0..hours_in_month {
            for lot_year in 2006usize..2026usize {
                let date_start = now + chrono::Duration::hours(next_hour);
                let date_end = now + chrono::Duration::hours(next_hour + 1);
                let cmd = CopartCmd::LotSearch {
                    page_number: 0,
                    date_start: date_start.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                    date_end: date_end.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                    year_start: lot_year,
                    year_end: lot_year,
                };

                if let Err(e) = self.cmd_sender.send(&cmd, &cmd.to_topic()).await {
                    error!("kafka message send failed: `{e}`")
                } else {
                    debug!(
                        "sent lot search command for page: 0, date: {}-{}, year: {}-{}",
                        date_start, date_end, lot_year, lot_year,
                    )
                }
            }
        }
        info!("lot search finished")
    }

    fn descriptor(&self) -> Option<&'static str> {
        Some("copart lot search")
    }
}

pub struct CopartAuctionJoinTask {
    cmd_sender: KafkaSender,
}

impl CopartAuctionJoinTask {
    pub fn new(cmd_sender: KafkaSender) -> Self {
        Self { cmd_sender }
    }
}

impl Default for CopartAuctionJoinTask {
    fn default() -> Self {
        Self::new(KafkaSender::new(CONFIG.kafka.url.to_owned()))
    }
}

#[async_trait]
impl Task for CopartAuctionJoinTask {
    async fn run(&self, opts: Option<&HashMap<String, String>>) {
        let cmd = CopartCmd::Auction("59-A".to_string());
        if let Err(e) = self.cmd_sender.send(&cmd, &cmd.to_topic()).await {
            error!("kafka message send failed: `{e}`")
        } else {
            debug!("sent auction command")
        }
    }

    fn descriptor(&self) -> Option<&'static str> {
        Some("copart auction join")
    }
}
