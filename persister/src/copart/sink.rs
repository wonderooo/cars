use crate::copart::CopartPersisterExt;
use common::io::copart::{CopartCmd, CopartResponse, LotSearchResponse};
use common::io::error::GeneralError;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{error, warn};

pub struct ExternalSignaling {
    pub cmd_sender: UnboundedSender<CopartResponse>,
    pub response_receiver: UnboundedReceiver<CopartCmd>,
}

pub struct CopartPersisterSink<P: CopartPersisterExt> {
    cmd_receiver: UnboundedReceiver<CopartResponse>,
    msg_handler: Arc<SingleMsgHandler<P>>,
}

struct SingleMsgHandler<P: CopartPersisterExt> {
    persister: P,
    response_sender: UnboundedSender<CopartCmd>,
}

impl<P: CopartPersisterExt> SingleMsgHandler<P> {
    async fn handle_message(&self, msg: CopartResponse) {
        match msg {
            CopartResponse::LotSearch(resp) => self.handle_lot_search(resp).await,
            CopartResponse::LotImageBlobs(resp) => {
                unimplemented!("lot image blobs response not implemented: `{resp:?}`");
            }
            CopartResponse::LotImages(resp) => {
                warn!(
                    "persister received lot images response, which should never happen: `{resp:?}`"
                )
            }
        }
    }

    async fn handle_lot_search(&self, incoming_msg: Result<LotSearchResponse, GeneralError>) {
        match incoming_msg {
            Ok(lsr) => {
                match self
                    .persister
                    .save_new_lot_vehicles(lsr.response.into())
                    .await
                {
                    Ok(lns) => lns.into_iter().for_each(|ln| {
                        let _ = self.response_sender.send(CopartCmd::LotImages(ln));
                    }),
                    Err(e) => error!(""),
                }
            }
            Err(e) => error!(""),
        }
    }
}

impl<P> CopartPersisterSink<P>
where
    P: CopartPersisterExt + Send + Sync + 'static,
{
    pub fn new(persister: P) -> (Self, ExternalSignaling) {
        let (cmd_sender, cmd_receiver) = tokio::sync::mpsc::unbounded_channel();
        let (response_sender, response_receiver) = tokio::sync::mpsc::unbounded_channel();
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
