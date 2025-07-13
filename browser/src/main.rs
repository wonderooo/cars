use browser::browser::{CopartBrowserCmd, CopartBrowserResponseVariant, Structured};
use browser::pool::CopartBrowserPool;
use common::kafka::{run_sender_on, KafkaReceiver, KafkaSender, SyncRxFn};
use common::logging::setup_logging;
use tokio_util::sync::CancellationToken;
use tracing::info;

#[tokio::main]
async fn main() {
    setup_logging("browser");
    let cancellation_token = CancellationToken::new();

    let ((cmd_sender, mut resp_receiver), pool_done) =
        CopartBrowserPool::<Structured>::run(8, ([127, 0, 0, 1], 8100), cancellation_token.clone())
            .await;

    let rx_done = KafkaReceiver::<CopartBrowserCmd, SyncRxFn>::new(
        "localhost:9092",
        "copart_cmd_lot_search_0",
        &["copart_cmd_lot_search", "copart_cmd_lot_images"],
        Box::new(move |x| {
            println!("{:?}", x);
            cmd_sender.send(x).unwrap()
        }),
        cancellation_token.clone(),
    )
    .run();

    let kafka_sender = KafkaSender::new("localhost:9092");
    let tx_done = run_sender_on(kafka_sender, cancellation_token.clone(), |s| {
        tokio::spawn(async move {
            while let Some(msg) = resp_receiver.recv().await {
                let topic = msg.inner.variant.topic();
                s.send_with_key(&msg, msg.inner.correlation_id.clone(), topic)
                    .await;
            }
        })
    });

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl c event");
    cancellation_token.cancel();
    info!("exiting");
    tokio::join!(rx_done.notified(), tx_done.notified(), pool_done.notified());
    info!("exited");
}
