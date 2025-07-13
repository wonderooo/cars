use crate::copart::response::{CopartRequesterResponse, LotImagesBlobResponse};
use crate::copart::CopartRequester;
use browser::browser::{CopartBrowserResponse, CopartBrowserResponseVariant, LotImagesResponse};
use browser::error::BrowserError;
use browser::pool::CopartBrowserPoolResponse;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::error;

pub struct ExternalSignaling {
    pub cmd_sender: UnboundedSender<CopartBrowserPoolResponse<CopartBrowserResponse>>,
    pub response_receiver: UnboundedReceiver<CopartRequesterResponse>,
}

pub struct InternalSignaling {
    pub cmd_receiver: UnboundedReceiver<CopartBrowserPoolResponse<CopartBrowserResponse>>,
    pub response_sender: UnboundedSender<CopartRequesterResponse>,
}

pub struct CopartRequesterSink {
    requester: CopartRequester,
    signaling: InternalSignaling,
}

impl CopartRequesterSink {
    pub fn new() -> (Self, ExternalSignaling) {
        let (cmd_sender, cmd_receiver) = tokio::sync::mpsc::unbounded_channel();
        let (response_sender, response_receiver) = tokio::sync::mpsc::unbounded_channel();
        let external_signaling = ExternalSignaling {
            cmd_sender,
            response_receiver,
        };
        let internal_signaling = InternalSignaling {
            cmd_receiver,
            response_sender,
        };
        let sink = Self {
            requester: CopartRequester::new(),
            signaling: internal_signaling,
        };
        (sink, external_signaling)
    }

    pub fn run(self, cancellation_token: CancellationToken) -> Arc<Notify> {
        let join_handle = tokio::spawn(self.run_blocking());

        let done = Arc::new(Notify::new());
        tokio::spawn({
            let done = done.clone();
            async move {
                cancellation_token.cancelled().await;
                join_handle.abort();
                done.notify_waiters();
            }
        });
        done
    }

    pub async fn run_blocking(mut self) {
        while let Some(msg) = self.signaling.cmd_receiver.recv().await {
            match msg.inner.variant {
                CopartBrowserResponseVariant::LotImages(result) => {
                    self.handle_lot_images(result).await;
                }
                _ => unreachable!("requester should only receive lot images responses"),
            }
        }
    }

    async fn handle_lot_images(&self, result: Result<LotImagesResponse, BrowserError>) {
        match result {
            Ok(lot_images) => {
                let images = self.requester.download_images(lot_images.response).await;
                self.signaling
                    .response_sender
                    .send(CopartRequesterResponse::LotImagesBlob(
                        LotImagesBlobResponse { images },
                    ))
                    .unwrap();
            }
            Err(e) => error!("error on processed copart lot images response: {e}"),
        }
    }
}
