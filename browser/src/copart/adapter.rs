use crate::copart::browser::CopartBrowserCmd;
use crate::copart::pool::CopartBrowserPoolResponse;
use async_trait::async_trait;
use common::kafka::{KafkaError, ReceiveHandle, SendHandle, SendMsg};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing::error;

pub struct CopartPoolTxKafkaAdapter {
    pub cmd_sender: UnboundedSender<CopartBrowserCmd>,
}

#[async_trait]
impl ReceiveHandle for CopartPoolTxKafkaAdapter {
    type RxItem = CopartBrowserCmd;

    async fn on_message(&self, maybe_msg: Result<Self::RxItem, KafkaError>) {
        match maybe_msg {
            Ok(msg) => self
                .cmd_sender
                .send(msg)
                .expect("tokio mpsc channel - cmd receiver is gone"),
            Err(e) => error!("kafka receive failed: `{e}`"),
        };
    }
}

pub struct CopartPoolRxKafkaAdapter {
    pub response_receiver: UnboundedReceiver<CopartBrowserPoolResponse>,
}

#[async_trait]
impl SendHandle for CopartPoolRxKafkaAdapter {
    type TxItem = CopartBrowserPoolResponse;

    async fn next(&mut self) -> Option<SendMsg<Self::TxItem>> {
        self.response_receiver.recv().await.map(|msg| SendMsg {
            topic: msg.inner.variant.topic().to_string(),
            msg,
        })
    }
}
