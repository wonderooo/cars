use chromiumoxide::Handler;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

pub struct BrowserEngineHandler {
    handler: Handler,
}

impl BrowserEngineHandler {
    pub fn new(handler: Handler) -> Self {
        Self { handler }
    }

    pub async fn handle_blocking(mut self) {
        while let Some(h) = self.handler.next().await {
            if let Err(e) = h {
                warn!("browser handler error: {:?}", e);
            }
        }
    }

    pub fn handle(self) -> JoinHandle<()> {
        tokio::spawn(self.handle_blocking())
    }

    pub fn handle_with_cancel(self, cancellation_token: CancellationToken) -> Arc<Notify> {
        let done = Arc::new(Notify::new());
        let join = tokio::spawn(self.handle_blocking());

        tokio::spawn({
            let done = done.clone();
            async move {
                cancellation_token.cancelled().await;
                join.abort();
                info!("browser engine handler exited");
                done.notify_waiters();
            }
        });

        done
    }
}
