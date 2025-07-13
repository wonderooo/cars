use common::logging::setup_logging;
use persister::copart::CopartPersister;
use tokio_util::sync::CancellationToken;
use tracing::info;

#[tokio::main]
async fn main() {
    setup_logging("persister");
    let cancellation_token = CancellationToken::new();

    let persister_done = CopartPersister::run(cancellation_token.clone());

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl c event");
    info!("exiting");
    cancellation_token.cancel();
    persister_done.notified().await;
    info!("exited");
}
