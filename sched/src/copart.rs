use crate::{minutes, Task};
use async_trait::async_trait;
use common::io::copart::CopartCmd;
use common::kafka::{KafkaSender, ToTopic};
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
        Self::new(KafkaSender::new("localhost:9092"))
    }
}

#[async_trait]
impl Task for CopartLotSearchTask {
    async fn run(&self) {
        for page in 0..150 {
            let cmd = CopartCmd::LotSearch(page);
            if let Err(e) = self.cmd_sender.send(&cmd, &cmd.to_topic()).await {
                error!("kafka message send failed: `{e}`")
            } else {
                debug!("sent lot search command for page: {page}")
            }
        }
        info!("lot search finished")
    }

    fn duration(&self) -> tokio::time::Duration {
        minutes(30)
    }

    fn descriptor(&self) -> Option<&'static str> {
        Some("copart lot search")
    }
}
