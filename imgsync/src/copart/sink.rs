use crate::copart::requester::{CopartRequesterExt, LotImageBlobsVector};
use crate::copart::uploader::CopartUploaderExt;
use common::io::copart::{CopartResponse, LotImagesResponse, LotNumber, SyncedImagesResponse};
use common::io::error::GeneralError;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{Notify, Semaphore};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, instrument, warn};

pub type MsgIn = CopartResponse;
pub type MsgOut = CopartResponse;

pub struct ExternalSignaling {
    pub cmd_sender: Sender<MsgIn>,
    pub response_receiver: Receiver<MsgOut>,
}

pub struct CopartImageSyncSink<R: CopartRequesterExt, U: CopartUploaderExt> {
    cmd_receiver: Receiver<MsgIn>,
    msg_handler: Arc<SingleMsgHandler<R, U>>,
    usage_permits: Arc<Semaphore>,
}

struct SingleMsgHandler<R: CopartRequesterExt, U: CopartUploaderExt> {
    requester: R,
    uploader: U,
    response_sender: Sender<MsgOut>,
}

pub struct LotImageBlobsResponse {
    pub lot_number: LotNumber,
    pub response: LotImageBlobsVector,
}

impl<R: CopartRequesterExt, U: CopartUploaderExt> SingleMsgHandler<R, U> {
    async fn handle_message(&self, msg: MsgIn) {
        match msg {
            MsgIn::LotImages(resp) => self.handle_lot_images(resp).await,
            MsgIn::LotSearch(_) => warn!(""),
            MsgIn::SyncedImages(_) => warn!(""),
        }
    }

    #[instrument(skip(self))]
    async fn handle_lot_images(&self, incoming_msg: Result<LotImagesResponse, GeneralError>) {
        match incoming_msg {
            Ok(images) => {
                let blobs = self.requester.download_images(images.response).await;
                let blobs_response = LotImageBlobsResponse {
                    lot_number: images.lot_number,
                    response: blobs,
                };

                let synced = self.uploader.upload_images(blobs_response.into()).await;
                let synced_response = SyncedImagesResponse {
                    lot_number: images.lot_number,
                    response: synced,
                };

                let _ = self
                    .response_sender
                    .send(MsgOut::SyncedImages(Ok(synced_response)))
                    .await;
            }
            Err(e) => error!(producer_error = ?e, "lot images response is an error"),
        }
    }
}

impl<R, U> CopartImageSyncSink<R, U>
where
    R: CopartRequesterExt + Send + Sync + 'static,
    U: CopartUploaderExt + Send + Sync + 'static,
{
    pub fn new(requester: R, uploader: U) -> (Self, ExternalSignaling) {
        let (cmd_sender, cmd_receiver) = tokio::sync::mpsc::channel(32);
        let (response_sender, response_receiver) = tokio::sync::mpsc::channel(32);
        let external_signaling = ExternalSignaling {
            cmd_sender,
            response_receiver,
        };
        let msg_handler = Arc::new(SingleMsgHandler {
            response_sender,
            requester,
            uploader,
        });
        let sink = Self {
            msg_handler,
            cmd_receiver,
            usage_permits: Arc::new(Semaphore::new(32)),
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

    #[instrument(skip(self))]
    pub async fn run_blocking(mut self) {
        while let Some(msg) = self.cmd_receiver.recv().await {
            debug!(incoming_msg = ?msg, "spawning handler for incoming message");
            let _permit = unsafe {
                self.usage_permits
                    .clone()
                    .acquire_owned()
                    .await
                    .unwrap_unchecked()
            };
            tokio::spawn({
                let handler = Arc::clone(&self.msg_handler);
                async move {
                    handler.handle_message(msg).await;
                    drop(_permit);
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::copart::requester::LotImageBlobs;
    use crate::copart::sink::{CopartImageSyncSink, MsgIn};
    use crate::copart::uploader::NewLotImages;
    use async_trait::async_trait;
    use common::io::copart::{LotImagesVector, SyncedImagesVector};
    use std::time::Duration;
    use tokio::time::Instant;

    struct NopCopartRequester;

    #[async_trait]
    impl CopartRequesterExt for NopCopartRequester {
        async fn download_images(&self, _cmds: LotImagesVector) -> LotImageBlobsVector {
            tokio::time::sleep(Duration::from_millis(20)).await;
            LotImageBlobsVector(vec![LotImageBlobs {
                standard: None,
                high_res: None,
                thumbnail: None,
                standard_url: None,
                high_res_url: None,
                thumbnail_url: None,
                sequence_number: 1,
                image_type: "jpg".to_string(),
            }])
        }
    }

    struct NopCopartUploader;

    #[async_trait]
    impl CopartUploaderExt for NopCopartUploader {
        async fn upload_images(&self, new_lot_images: NewLotImages) -> SyncedImagesVector {
            todo!()
        }
    }

    #[tokio::test]
    async fn test_sink_concurrency() -> Result<(), Box<dyn std::error::Error>> {
        let (sink, mut sig) = CopartImageSyncSink::new(NopCopartRequester, NopCopartUploader);
        tokio::spawn(sink.run_blocking());

        for _ in 0..16 {
            sig.cmd_sender
                .send(MsgIn::LotImages(Ok(LotImagesResponse {
                    lot_number: 69,
                    response: LotImagesVector(vec![]),
                })))
                .await?;
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
