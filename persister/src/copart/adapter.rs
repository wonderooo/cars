use async_trait::async_trait;
use common::io::copart::{CopartCmd, CopartResponse};
use common::kafka::{KafkaError, ReceiveHandle, SendHandle, SendMsg, ToTopic};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing::error;

pub struct CopartSinkTxKafkaAdapter {
    pub cmd_sender: UnboundedSender<CopartResponse>,
}

#[async_trait]
impl ReceiveHandle for CopartSinkTxKafkaAdapter {
    type RxItem = CopartResponse;

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

pub struct CopartSinkRxKafkaAdapter {
    pub response_receiver: UnboundedReceiver<CopartCmd>,
}

#[async_trait]
impl SendHandle for CopartSinkRxKafkaAdapter {
    type TxItem = CopartCmd;

    async fn next(&mut self) -> Option<SendMsg<Self::TxItem>> {
        self.response_receiver.recv().await.map(|msg| SendMsg {
            topic: msg.to_topic(),
            msg,
        })
    }
}
