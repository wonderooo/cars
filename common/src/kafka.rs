use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::ToBytes;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::{ClientConfig, Message};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::field::debug;
use tracing::{debug, error, info};
use uuid::Uuid;

pub type BoxedAsyncFn<A, R> = Box<dyn Fn(A) -> Pin<Box<dyn Future<Output = R> + Send>> + Send>;
pub type BoxedFn<A, R> = Box<dyn (Fn(A) -> R) + Send>;
pub struct AsyncRxFn;
pub struct SyncRxFn;

pub trait RxFn<A, R> {
    type Fn;

    fn call(a: A, f: &Self::Fn) -> Pin<Box<dyn Future<Output = R> + Send>>;
}

impl<A, R> RxFn<A, R> for AsyncRxFn {
    type Fn = BoxedAsyncFn<A, R>;

    fn call(a: A, f: &Self::Fn) -> Pin<Box<dyn Future<Output = R> + Send>> {
        f(a)
    }
}

impl<A, R: Send + 'static> RxFn<A, R> for SyncRxFn {
    type Fn = BoxedFn<A, R>;

    fn call(a: A, f: &Self::Fn) -> Pin<Box<dyn Future<Output = R> + Send>> {
        let r = f(a);
        Box::pin(async move { r })
    }
}

pub struct KafkaReceiver<R, F>
where
    R: DeserializeOwned + 'static,
    F: RxFn<R, ()>,
{
    consumer: StreamConsumer,
    on_rx: F::Fn,
    cancellation_token: CancellationToken,
    phantom: PhantomData<fn(R)>,
}

impl<R, F> KafkaReceiver<R, F>
where
    R: DeserializeOwned,
    F: RxFn<R, ()>,
{
    pub fn new(
        bootstrap_server: impl Into<String>,
        consumer_group: impl Into<String>,
        topics: &[&str],
        on_rx: F::Fn,
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
            on_rx,
            cancellation_token,
            phantom: PhantomData,
        }
    }

    pub fn run(self) -> Arc<Notify>
    where
        <F as RxFn<R, ()>>::Fn: Send,
        <F as RxFn<R, ()>>::Fn: 'static,
    {
        let join_handle = tokio::spawn(async move {
            Self::consume(self.consumer, self.on_rx).await;
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

    async fn consume(queue_consumer: StreamConsumer, on_rx: F::Fn) {
        while let Ok(m) = queue_consumer.recv().await {
            let payload = match m.payload_view::<str>() {
                None => "",
                Some(Ok(s)) => s,
                Some(Err(e)) => {
                    println!("Error while deserializing message payload: {:?}", e);
                    ""
                }
            };

            let unmarshalled =
                serde_json::from_str::<R>(payload).expect("Can't deserialize message payload");
            F::call(unmarshalled, &on_rx).await;

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
