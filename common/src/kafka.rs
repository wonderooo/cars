use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::ToBytes;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::{ClientConfig, Message};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};
use uuid::Uuid;

pub trait ReceiveHandle {
    type RxItem: DeserializeOwned;

    fn on_message(&self, msg: Self::RxItem) -> impl Future<Output = ()> + Send;
}

pub struct KafkaReceiver<H> {
    consumer: StreamConsumer,
    receive_handle: H,
    cancellation_token: CancellationToken,
}

impl<H> KafkaReceiver<H>
where
    H: ReceiveHandle + Send + 'static,
{
    pub fn new(
        bootstrap_server: impl Into<String>,
        consumer_group: impl Into<String>,
        topics: &[&str],
        receive_handle: H,
        cancellation_token: CancellationToken,
    ) -> Self {
        let mut config = ClientConfig::new();

        config
            .set("group.id", consumer_group)
            .set("bootstrap.servers", bootstrap_server)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .set_log_level(RDKafkaLogLevel::Debug);
        let consumer: StreamConsumer = config.create().expect("consumer creation failed");
        consumer
            .subscribe(topics)
            .expect("can't subscribe to specified topics");

        Self {
            consumer,
            receive_handle,
            cancellation_token,
        }
    }

    pub fn run(self) -> Arc<Notify> {
        let join_handle = tokio::spawn(async move {
            Self::consume(self.consumer, self.receive_handle).await;
        });

        let done = Arc::new(Notify::new());
        tokio::spawn({
            let done = Arc::clone(&done);
            async move {
                self.cancellation_token.cancelled().await;
                join_handle.abort_handle().abort();
                info!("kafka cmd receiver closed");
                done.notify_waiters();
            }
        });

        done
    }

    async fn consume(queue_consumer: StreamConsumer, receive_handle: H) {
        while let Ok(m) = queue_consumer.recv().await {
            let payload = match m.payload_view::<str>() {
                None => "",
                Some(Ok(s)) => s,
                Some(Err(e)) => {
                    println!("Error while deserializing message payload: {:?}", e);
                    ""
                }
            };

            let unmarshalled = serde_json::from_str::<H::RxItem>(payload)
                .expect("Can't deserialize message payload");
            receive_handle.on_message(unmarshalled).await;

            queue_consumer
                .commit_message(&m, CommitMode::Async)
                .unwrap();
        }
    }
}

pub struct KafkaSender {
    producer: FutureProducer,
}

impl KafkaSender {
    pub fn new(bootstrap_server: impl Into<String>) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_server)
            .set("message.max.bytes", "100000000")
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");

        Self { producer }
    }

    pub async fn send<R: Serialize>(&self, msg: &R, topic: &str) {
        let id = Uuid::new_v4().as_simple().to_string();
        self.send_with_key(msg, id, topic).await;
    }

    pub async fn send_with_key<R: Serialize>(&self, msg: &R, key: impl ToBytes, topic: &str) {
        let marshalled = serde_json::to_string(&msg).unwrap();
        let queue_msg = FutureRecord::to(topic).payload(&marshalled).key(&key);

        let status = self.producer.send(queue_msg, Duration::from_secs(0)).await;
        match status {
            Err((e, _)) => error!("failed to send message to kafka queue: {:?}", e),
            Ok((partition, offset)) => {
                debug!("sent kafka message to partition `{partition}`, offset `{offset}`")
            }
        }
    }
}

pub fn run_sender_on(
    sender: KafkaSender,
    cancellation_token: CancellationToken,
    on: impl FnOnce(KafkaSender) -> JoinHandle<()>,
) -> Arc<Notify> {
    let join_handle = on(sender);

    let done = Arc::new(Notify::new());
    tokio::spawn({
        let done = Arc::clone(&done);
        async move {
            cancellation_token.cancelled().await;
            join_handle.abort_handle().abort();
            info!("kafka sender closed");
            done.notify_waiters();
        }
    });

    done
}
