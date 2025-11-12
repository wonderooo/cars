use async_trait::async_trait;
use common::io::copart::{CopartCmd, CopartResponse};
use common::kafka::{KafkaError, ReceiveHandle, SendHandle, SendMsg, ToTopic};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::error;

pub struct CopartPoolTxKafkaAdapter {
    pub cmd_sender: Sender<CopartCmd>,
}

#[async_trait]
impl ReceiveHandle for CopartPoolTxKafkaAdapter {
    type RxItem = CopartCmd;

    async fn on_message(&self, maybe_msg: Result<Self::RxItem, KafkaError>) {
        match maybe_msg {
            Ok(msg) => self
                .cmd_sender
                .send(msg)
                .await
                .expect("tokio mpsc channel - cmd receiver is gone"),
            Err(e) => error!("kafka receive failed: `{e}`"),
        };
    }
}

pub struct CopartPoolRxKafkaAdapter {
    pub response_receiver: Receiver<CopartResponse>,
}

#[async_trait]
impl SendHandle for CopartPoolRxKafkaAdapter {
    type TxItem = CopartResponse;

    async fn next(&mut self) -> Option<SendMsg<Self::TxItem>> {
        self.response_receiver.recv().await.map(|msg| SendMsg {
            topic: msg.to_topic(),
            msg,
        })
    }
}
