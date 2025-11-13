use browser::copart::adapter::{CopartPoolRxKafkaAdapter, CopartPoolTxKafkaAdapter};
use browser::copart::pool::CopartBrowserPool;
use common::config::CONFIG;
use common::kafka::{KafkaReceiver, KafkaSender};
use common::logging::setup_logging;
use tokio_util::sync::CancellationToken;
use tracing::info;

#[tokio::main]
async fn main() {
    setup_logging("browser");
    info!("starting app");
    let cancellation_token = CancellationToken::new();

    let (pool, sig) = CopartBrowserPool::new(
        CONFIG.proxy.host.to_owned(),
        CONFIG.proxy.port,
        cancellation_token.clone(),
    );
    let pool_done = pool.run(4).await;

    let rx_done = KafkaReceiver::new(
        CONFIG.kafka.url.to_owned(),
        "copart_cmd_lot_search_0",
        &[
            "copart_cmd_lot_search",
            "copart_cmd_lot_images",
            "copart_cmd_auction",
            "copart_cmd_login_refresh",
        ],
    )
    .run_on(
        CopartPoolTxKafkaAdapter {
            cmd_sender: sig.cmd_sender,
        },
        cancellation_token.clone(),
    );

    let tx_done = KafkaSender::new(CONFIG.kafka.url.to_owned()).run_on(
        CopartPoolRxKafkaAdapter {
            response_receiver: sig.response_receiver,
        },
        cancellation_token.clone(),
    );

    info!("app started");
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl c event");
    cancellation_token.cancel();
    info!("exiting");
    tokio::join!(rx_done.notified(), tx_done.notified(), pool_done.notified());
    info!("exited");
}
