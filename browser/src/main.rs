use browser::copart::adapter::{CopartPoolRxKafkaAdapter, CopartPoolTxKafkaAdapter};
use browser::copart::pool::CopartBrowserPool;
use common::kafka::{KafkaReceiver, KafkaSender};
use common::logging::setup_logging;
use tokio_util::sync::CancellationToken;
use tracing::info;

#[tokio::main]
async fn main() {
    setup_logging("browser");
    let cancellation_token = CancellationToken::new();

    let ((cmd_sender, response_receiver), pool_done) =
        CopartBrowserPool::run(8, ([127, 0, 0, 1], 8100), cancellation_token.clone()).await;

    let rx_done = KafkaReceiver::new(
        "localhost:9092",
        "copart_cmd_lot_search_0",
        &["copart_cmd_lot_search", "copart_cmd_lot_images"],
    )
    .run_on(
        CopartPoolTxKafkaAdapter { cmd_sender },
        cancellation_token.clone(),
    );

    let tx_done = KafkaSender::new("localhost:9092").run_on(
        CopartPoolRxKafkaAdapter { response_receiver },
        cancellation_token.clone(),
    );

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl c event");
    cancellation_token.cancel();
    info!("exiting");
    tokio::join!(rx_done.notified(), tx_done.notified(), pool_done.notified());
    info!("exited");
}
