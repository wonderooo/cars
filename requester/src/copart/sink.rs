use crate::copart::client::CopartRequesterExt;
use common::io::copart::{CopartResponse, LotImageBlobsResponse, LotImagesResponse};
use common::io::error::GeneralError;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::warn;

pub type MsgIn = CopartResponse;
pub type MsgOut = CopartResponse;

pub struct ExternalSignaling {
    pub cmd_sender: UnboundedSender<MsgIn>,
    pub response_receiver: UnboundedReceiver<MsgOut>,
}

pub struct CopartRequesterSink<R: CopartRequesterExt> {
    cmd_receiver: UnboundedReceiver<MsgIn>,
    msg_handler: Arc<SingleMsgHandler<R>>,
}

struct SingleMsgHandler<R: CopartRequesterExt> {
    requester: R,
    response_sender: UnboundedSender<MsgOut>,
}

impl<R: CopartRequesterExt> SingleMsgHandler<R> {
    async fn handle_message(&self, msg: MsgIn) {
        match msg {
            MsgIn::LotImages(resp) => self.handle_lot_images(resp).await,
            MsgIn::LotImageBlobs(_) => warn!(""),
            MsgIn::LotSearch(_) => warn!(""),
        }
    }

    async fn handle_lot_images(&self, incoming_msg: Result<LotImagesResponse, GeneralError>) {
        match incoming_msg {
            Ok(images) => {
                let blobs = self.requester.download_images(images.response).await;
                let _ =
                    self.response_sender
                        .send(MsgOut::LotImageBlobs(Ok(LotImageBlobsResponse {
                            lot_number: images.lot_number,
                            response: blobs,
                        })));
            }
            Err(e) => {}
        }
    }
}

impl<R> CopartRequesterSink<R>
where
    R: CopartRequesterExt + Send + Sync + 'static,
{
    pub fn new(requester: R) -> (Self, ExternalSignaling) {
        let (cmd_sender, cmd_receiver) = tokio::sync::mpsc::unbounded_channel();
        let (response_sender, response_receiver) = tokio::sync::mpsc::unbounded_channel();
        let external_signaling = ExternalSignaling {
            cmd_sender,
            response_receiver,
        };
        let msg_handler = Arc::new(SingleMsgHandler {
            response_sender,
            requester,
        });
        let sink = Self {
            msg_handler,
            cmd_receiver,
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
        while let Some(msg) = self.cmd_receiver.recv().await {
            tokio::spawn({
                let handler = Arc::clone(&self.msg_handler);
                async move {
                    handler.handle_message(msg).await;
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::copart::sink::{CopartRequesterSink, MsgIn};
    use async_trait::async_trait;
    use common::io::copart::{LotImageBlobs, LotImages};
    use std::time::Duration;
    use tokio::time::Instant;

    struct NopCopartRequester;

    #[async_trait]
    impl CopartRequesterExt for NopCopartRequester {
        async fn download_images(&self, _cmds: Vec<LotImages>) -> Vec<LotImageBlobs> {
            tokio::time::sleep(Duration::from_millis(20)).await;
            vec![LotImageBlobs {
                standard: None,
                high_res: None,
                thumbnail: None,
            }]
        }
    }

    #[tokio::test]
    async fn test_sink_concurrency() -> Result<(), Box<dyn std::error::Error>> {
        let (sink, mut sig) = CopartRequesterSink::new(NopCopartRequester);
        tokio::spawn(sink.run_blocking());

        for _ in 0..16 {
            sig.cmd_sender.send(MsgIn::LotImages(Ok(LotImagesResponse {
                lot_number: 69,
                response: vec![],
            })))?;
        }

        let mut responses = vec![];
        let start = Instant::now();
        for _ in 0..16 {
            let resp = sig.response_receiver.recv().await.ok_or("recv error")?;
            responses.push(resp);
        }

        assert_eq!(start.elapsed().as_millis() < 25, true);
        assert_eq!(responses.len(), 16);

        Ok(())
    }
}
