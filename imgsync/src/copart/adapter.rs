use crate::copart::sink::{MsgIn, MsgOut};
use async_trait::async_trait;
use common::kafka::{KafkaError, ReceiveHandle, SendHandle, SendMsg, ToTopic};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::error;

pub struct CopartSinkTxKafkaAdapter {
    pub cmd_sender: Sender<MsgIn>,
}

#[async_trait]
impl ReceiveHandle for CopartSinkTxKafkaAdapter {
    type RxItem = MsgIn;

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

pub struct CopartSinkRxKafkaAdapter {
    pub response_receiver: Receiver<MsgOut>,
}

#[async_trait]
impl SendHandle for CopartSinkRxKafkaAdapter {
    type TxItem = MsgOut;

    async fn next(&mut self) -> Option<SendMsg<Self::TxItem>> {
        self.response_receiver.recv().await.map(|msg| SendMsg {
            topic: msg.to_topic(),
            msg,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::io::copart::{
        LotImagesResponse, LotImagesVector, SyncedImagesResponse, SyncedImagesVector,
    };
    use common::kafka::{KafkaAdmin, KafkaReceiver, KafkaSender};
    use testcontainers_modules::kafka::apache;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;

    #[tokio::test]
    async fn test_tx_adapter() -> Result<(), Box<dyn std::error::Error>> {
        let container = apache::Kafka::default().start().await?;
        let kafka_port = container.get_host_port_ipv4(apache::KAFKA_PORT).await?;
        let kafka_addr = format!("127.0.0.1:{kafka_port}");

        let (cmd_sender, mut cmd_receiver) = tokio::sync::mpsc::channel(32);
        let tx_adapter = CopartSinkTxKafkaAdapter { cmd_sender };

        KafkaAdmin::new(&kafka_addr)
            .create_topic("copart_response_synced_images")
            .await?;
        tokio::spawn(
            KafkaReceiver::new(
                &kafka_addr,
                "test_group",
                &["copart_response_synced_images"],
            )
            .run_on_blocking(tx_adapter),
        );
        let sender = KafkaSender::new(&kafka_addr);
        sender
            .send(
                &MsgIn::LotImages(Ok(LotImagesResponse {
                    lot_number: 69,
                    response: LotImagesVector(vec![]),
                })),
                "copart_response_synced_images",
            )
            .await?;

        assert!(cmd_receiver.recv().await.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_rx_adapter() -> Result<(), Box<dyn std::error::Error>> {
        let container = apache::Kafka::default().start().await?;
        let kafka_port = container.get_host_port_ipv4(apache::KAFKA_PORT).await?;
        let kafka_addr = format!("127.0.0.1:{kafka_port}");

        let (response_sender, response_receiver) = tokio::sync::mpsc::channel(32);
        let rx_adapter = CopartSinkRxKafkaAdapter { response_receiver };

        KafkaAdmin::new(&kafka_addr)
            .create_topic("copart_response_synced_images")
            .await?;
        tokio::spawn(KafkaSender::new(&kafka_addr).run_on_blocking(rx_adapter));
        response_sender
            .send(MsgOut::SyncedImages(Ok(SyncedImagesResponse {
                lot_number: 69,
                response: SyncedImagesVector(vec![]),
            })))
            .await?;

        assert!(
            KafkaReceiver::new(
                &kafka_addr,
                "test_group",
                &["copart_response_synced_images"]
            )
            .recv::<MsgOut>()
            .await
            .is_ok()
        );
        Ok(())
    }
}
