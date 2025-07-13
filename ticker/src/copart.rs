use crate::{Scheduler, Task};
use browser::browser::{CopartBrowserCmd, CopartBrowserCmdVariant};
use common::kafka::KafkaSender;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};
use uuid::Uuid;

pub struct CopartTicker {
    cmd_sender: KafkaSender,
    scheduler: Scheduler,
}

impl CopartTicker {
    pub fn new(cancellation_token: CancellationToken) -> Self {
        Self {
            cmd_sender: KafkaSender::new("localhost:9092"),
            scheduler: Scheduler::new(vec![], cancellation_token),
        }
    }

    pub fn run(mut self) -> Arc<Notify> {
        let cmd_sender = Arc::new(self.cmd_sender);

        self.scheduler.add_task(Task::new(
            Duration::from_secs(60 * 15),
            Box::new(move || Box::pin(Self::lot_search(Arc::clone(&cmd_sender)))),
        ));
        self.scheduler.run()
    }

    async fn lot_search(cmd_sender: impl AsRef<KafkaSender>) {
        for page in 0..150 {
            let correlation_id = Uuid::new_v4();
            cmd_sender
                .as_ref()
                .send_with_key(
                    &CopartBrowserCmd {
                        correlation_id: correlation_id.as_simple().to_string(),
                        variant: CopartBrowserCmdVariant::LotSearch(page),
                    },
                    correlation_id.as_simple().to_string(),
                    "copart_cmd_lot_search",
                )
                .await;
            debug!("sent lot search command for page: {page}")
        }
        info!("lot search finished")
    }
}
