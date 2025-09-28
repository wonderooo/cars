use crate::bucket::models::NewLotImages;
use crate::copart::CopartPersisterExt;
use common::io::copart::{CopartCmd, CopartResponse, LotImageBlobsResponse, LotSearchResponse};
use common::io::error::GeneralError;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{Notify, Semaphore};
use tokio_util::sync::CancellationToken;
use tracing::{error, instrument, warn};

pub struct ExternalSignaling {
    pub cmd_sender: Sender<CopartResponse>,
    pub response_receiver: Receiver<CopartCmd>,
}

pub struct CopartPersisterSink<P: CopartPersisterExt> {
    cmd_receiver: Receiver<CopartResponse>,
    msg_handler: Arc<SingleMsgHandler<P>>,
    usage_permit: Arc<Semaphore>,
}

struct SingleMsgHandler<P: CopartPersisterExt> {
    persister: P,
    response_sender: Sender<CopartCmd>,
}

impl<P: CopartPersisterExt> SingleMsgHandler<P> {
    async fn handle_message(&self, msg: CopartResponse) {
        match msg {
            CopartResponse::LotSearch(resp) => self.handle_lot_search(resp).await,
            CopartResponse::LotImageBlobs(resp) => self.handle_lot_image_blobs(resp).await,
            CopartResponse::LotImages(resp) => {
                warn!(
                    "persister received lot images response, which should never happen: `{resp:?}`"
                )
            }
        }
    }

    #[instrument(skip(self))]
    async fn handle_lot_search(&self, incoming_msg: Result<LotSearchResponse, GeneralError>) {
        match incoming_msg {
            Ok(lsr) => {
                match self
                    .persister
                    .save_new_lot_vehicles(lsr.response.into())
                    .await
                {
                    Ok(lns) => {
                        futures::stream::iter(lns)
                            .for_each(|ln| async move {
                                let _ = self.response_sender.send(CopartCmd::LotImages(ln)).await;
                            })
                            .await
                    }
                    Err(e) => error!(persister_error = ?e, "save new lot vehicles failed"),
                }
            }
            Err(e) => {
                error!(producer_error = ?e, "lot search response in an error")
            }
        }
    }

    #[instrument(skip(self))]
    async fn handle_lot_image_blobs(
        &self,
        incoming_msg: Result<LotImageBlobsResponse, GeneralError>,
    ) {
        match incoming_msg {
            Ok(blobs_resp) => {
                let new_lot_images: NewLotImages = blobs_resp.into();
                match self.persister.save_new_lot_images(new_lot_images).await {
                    Ok(_lns) => {}
                    Err(e) => error!(persister_error = ?e, "save new lot images failed"),
                }
            }
            Err(e) => error!(producer_error = ?e, "lot image blobs response in an error"),
        }
    }
}

impl<P> CopartPersisterSink<P>
where
    P: CopartPersisterExt + Send + Sync + 'static,
{
    pub fn new(persister: P) -> (Self, ExternalSignaling) {
        let (cmd_sender, cmd_receiver) = tokio::sync::mpsc::channel(32);
        let (response_sender, response_receiver) = tokio::sync::mpsc::channel(32);
        let external_signaling = ExternalSignaling {
            cmd_sender,
            response_receiver,
        };
        let msg_handler = Arc::new(SingleMsgHandler {
            response_sender,
            persister,
        });
        let sink = Self {
            msg_handler,
            cmd_receiver,
            usage_permit: Arc::new(Semaphore::new(32)),
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
            let _permit = unsafe {
                self.usage_permit
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
