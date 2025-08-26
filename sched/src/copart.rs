use crate::{minutes, Task};
use async_trait::async_trait;
use browser::copart::browser::{CopartBrowserCmd, CopartBrowserCmdVariant};
use common::kafka::KafkaSender;
use tracing::{debug, error, info};
use uuid::Uuid;

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
            let correlation_id = Uuid::new_v4();
            if let Err(e) = self
                .cmd_sender
                .send_with_key(
                    &CopartBrowserCmd {
                        correlation_id: correlation_id.as_simple().to_string(),
                        variant: CopartBrowserCmdVariant::LotSearch(page),
                    },
                    correlation_id.as_simple().to_string(),
                    "copart_cmd_lot_search",
                )
                .await
            {
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
