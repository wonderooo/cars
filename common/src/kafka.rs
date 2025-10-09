use async_trait::async_trait;
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::error::RDKafkaErrorCode;
use rdkafka::message::ToBytes;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use rdkafka::{ClientConfig, Message};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};
use uuid::Uuid;

pub struct KafkaAdmin {
    client: AdminClient<DefaultClientContext>,
}

impl KafkaAdmin {
    pub fn new(bootstrap_server: impl Into<String>) -> Self {
        Self {
            client: ClientConfig::new()
                .set("bootstrap.servers", bootstrap_server)
                .create()
                .expect("admin creation failed"),
        }
    }

    pub async fn create_topic_with_options(
        &self,
        topic: &str,
        opts: &HashMap<&str, &str>,
    ) -> Result<(), KafkaError> {
        let new_topic = NewTopic::new(topic, 1, TopicReplication::Fixed(1));
        let new_topic = opts.iter().fold(new_topic, |acc, (k, v)| acc.set(k, v));
        let results = self
            .client
            .create_topics(&[new_topic], &AdminOptions::default())
            .await?;

        // SAFETY: client must produce one topic result
        let result = unsafe { results.into_iter().next().unwrap_unchecked() };
        Ok(result.map(|_| ())?)
    }

    pub async fn create_topic(&self, topic: &str) -> Result<(), KafkaError> {
        self.create_topic_with_options(topic, &HashMap::new()).await
    }

    pub async fn create_absent_topic(&self, topic: &str) -> Result<bool, KafkaError> {
        if !self.topic_exist(topic).await? {
            self.create_topic(topic).await?;
            return Ok(true);
        }
        Ok(false)
    }

    pub async fn create_absent_topic_with_opts(
        &self,
        topic: &str,
        opts: &HashMap<&str, &str>,
    ) -> Result<bool, KafkaError> {
        if !self.topic_exist(topic).await? {
            self.create_topic_with_options(topic, opts).await?;
            return Ok(true);
        }
        Ok(false)
    }

    pub async fn delete_topic(&self, topic: &str) -> Result<(), KafkaError> {
        let results = self
            .client
            .delete_topics(&[topic], &AdminOptions::default())
            .await?;

        // SAFETY: client must produce one topic result
        let result = unsafe { results.into_iter().next().unwrap_unchecked() };
        Ok(result.map(|_| ())?)
    }

    pub async fn recreate_topic_with_opts(
        &self,
        topic: &str,
        opts: &HashMap<&str, &str>,
    ) -> Result<(), KafkaError> {
        self.delete_topic(topic).await?;
        self.create_topic_with_options(topic, opts).await
    }

    pub async fn recreate_topic(&self, topic: &str) -> Result<(), KafkaError> {
        self.recreate_topic_with_opts(topic, &HashMap::new()).await
    }

    pub async fn topic_exist(&self, topic: &str) -> Result<bool, KafkaError> {
        let meta = self.client.inner().fetch_metadata(None, Timeout::Never)?;
        Ok(meta.topics().iter().any(|t| t.name() == topic))
    }
}

#[async_trait]
pub trait ReceiveHandle {
    type RxItem: DeserializeOwned + Send;

    async fn on_message(&self, msg: Result<Self::RxItem, KafkaError>);
}

pub struct KafkaReceiver {
    consumer: StreamConsumer,
}

impl KafkaReceiver {
    pub fn new(
        bootstrap_server: impl Into<String>,
        consumer_group: impl Into<String>,
        topics: &[&str],
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

        Self { consumer }
    }

    pub async fn recv<R: DeserializeOwned>(&self) -> Result<R, KafkaError> {
        let raw = self.consumer.recv().await?;
        self.consumer.commit_message(&raw, CommitMode::Async)?;
        let msg = raw
            .payload_view::<str>()
            .ok_or(KafkaError::EmptyPayload)??;
        Ok(serde_json::from_str::<R>(msg)?)
    }

    pub async fn run_on_blocking<H>(self, receive_handle: H)
    where
        H: ReceiveHandle,
    {
        loop {
            receive_handle.on_message(self.recv().await).await;
        }
    }

    pub fn run_on<H>(self, receive_handle: H, cancellation_token: CancellationToken) -> Arc<Notify>
    where
        H: ReceiveHandle + Send + Sync + 'static,
    {
        let join_handle = tokio::spawn(self.run_on_blocking(receive_handle));

        let done = Arc::new(Notify::new());
        tokio::spawn({
            let done = Arc::clone(&done);
            async move {
                cancellation_token.cancelled().await;
                join_handle.abort_handle().abort();
                info!("kafka cmd receiver closed");
                done.notify_waiters();
            }
        });

        done
    }
}

pub struct SendMsg<S: Serialize> {
    pub msg: S,
    pub topic: String,
}

#[async_trait]
pub trait SendHandle {
    type TxItem: Serialize + Send + Sync;
    async fn next(&mut self) -> Option<SendMsg<Self::TxItem>>;
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

    pub async fn send<R: Serialize>(&self, msg: &R, topic: &str) -> Result<(), KafkaError> {
        let id = Uuid::new_v4().as_simple().to_string();
        self.send_with_key(msg, id, topic).await
    }

    pub async fn send_with_key<R: Serialize>(
        &self,
        msg: &R,
        key: impl ToBytes,
        topic: &str,
    ) -> Result<(), KafkaError> {
        let marshalled = serde_json::to_string(&msg)?;
        let queue_msg = FutureRecord::to(topic).payload(&marshalled).key(&key);

        let delivery = self
            .producer
            .send(queue_msg, Duration::from_secs(0))
            .await
            .map_err(|(e, _)| e)?;

        debug!(
            "sent kafka message to partition `{}`, offset `{}`",
            delivery.partition, delivery.offset
        );
        Ok(())
    }

    pub async fn run_on_blocking<H>(self, mut send_handle: H)
    where
        H: SendHandle,
    {
        while let Some(send_msg) = send_handle.next().await {
            if let Err(e) = self.send(&send_msg.msg, &send_msg.topic).await {
                error!("kafka message send failed: `{e}`");
            }
        }
    }

    pub fn run_on<H>(self, send_handle: H, cancellation_token: CancellationToken) -> Arc<Notify>
    where
        H: SendHandle + Send + 'static,
    {
        let join_handle = tokio::spawn(self.run_on_blocking(send_handle));

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
}

pub trait ToTopic {
    fn to_topic(&self) -> String;
}

#[derive(Debug, Error)]
pub enum KafkaError {
    #[error("kafka topic creation failed with code: `{0}`")]
    TopicCreate(RDKafkaErrorCode),
    #[error("received no bytes from kafka stream")]
    EmptyPayload,
    #[error("failed to convert kafka message to string: `{0}`")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("rdkafka error: `{0}`")]
    Rd(#[from] rdkafka::error::KafkaError),
    #[error("failed to serialize/deserialize kafka message: `{0}`")]
    Json(#[from] serde_json::Error),
}

impl From<(String, RDKafkaErrorCode)> for KafkaError {
    fn from((_, code): (String, RDKafkaErrorCode)) -> Self {
        Self::TopicCreate(code)
    }
}
