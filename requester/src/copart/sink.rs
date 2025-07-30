use crate::copart::client::ICopartRequester;
use crate::copart::io::{CopartImageBlobCmd, CopartRequesterCmd, CopartRequesterResponse};
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

pub type MsgIn = CopartRequesterCmd;
pub type MsgOut = CopartRequesterResponse;

pub struct ExternalSignaling {
    pub cmd_sender: UnboundedSender<MsgIn>,
    pub response_receiver: UnboundedReceiver<MsgOut>,
}

pub struct CopartRequesterSink<R: ICopartRequester> {
    cmd_receiver: UnboundedReceiver<MsgIn>,
    msg_handler: Arc<SingleMsgHandler<R>>,
}

struct SingleMsgHandler<R: ICopartRequester> {
    requester: R,
    response_sender: UnboundedSender<MsgOut>,
}

impl<R: ICopartRequester> SingleMsgHandler<R> {
    async fn handle_message(&self, msg: MsgIn) {
        match msg {
            CopartRequesterCmd::LotImageBlobs { cmds } => self.handle_lot_images(cmds).await,
        }
    }

    async fn handle_lot_images(&self, cmds: Vec<CopartImageBlobCmd>) {
        let images = self.requester.download_images(cmds).await;
        self.response_sender
            .send(CopartRequesterResponse::LotImageBlobs { images })
            .unwrap();
    }
}

impl<R> CopartRequesterSink<R>
where
    R: ICopartRequester + Send + Sync + 'static,
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
