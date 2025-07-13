use common::logging::setup_logging;
use ticker::copart::CopartTicker;
use tokio_util::sync::CancellationToken;
use tracing::info;

#[tokio::main]
async fn main() {
    setup_logging("ticker");

    let cancellation_token = CancellationToken::new();
    let copart_ticker_done = CopartTicker::new(cancellation_token.clone()).run();

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl c event");
    cancellation_token.cancel();
    info!("exiting");
    tokio::join!(copart_ticker_done.notified());
    info!("exited");
}
