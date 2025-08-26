use common::kafka::{KafkaReceiver, KafkaSender};
use common::logging::setup_logging;
use requester::copart::adapter::{CopartSinkRxKafkaAdapter, CopartSinkTxKafkaAdapter};
use requester::copart::client::CopartRequester;
use requester::copart::sink::CopartRequesterSink;
use tokio_util::sync::CancellationToken;
use tracing::info;

#[tokio::main]
async fn main() {
    setup_logging("requester");
    let cancellation_token = CancellationToken::new();

    let (copart_sink, copart_sig) = CopartRequesterSink::new(CopartRequester::new());
    let copart_sink_done = copart_sink.run(cancellation_token.clone());
    let rx_done = KafkaReceiver::new(
        "localhost:9092",
        "copart_response_lot_images_0",
        &["copart_response_lot_images"],
    )
    .run_on(
        CopartSinkTxKafkaAdapter {
            cmd_sender: copart_sig.cmd_sender,
        },
        cancellation_token.clone(),
    );

    let tx_done = KafkaSender::new("localhost:9092").run_on(
        CopartSinkRxKafkaAdapter {
            response_receiver: copart_sig.response_receiver,
        },
        cancellation_token.clone(),
    );

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl c event");
    info!("exiting");
    cancellation_token.cancel();
    tokio::join!(
        tx_done.notified(),
        rx_done.notified(),
        copart_sink_done.notified()
    );
    info!("exited");
}
