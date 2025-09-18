use common::kafka::{KafkaAdmin, KafkaReceiver, KafkaSender};
use common::logging::setup_logging;
use persister::copart::adapter::{CopartSinkRxKafkaAdapter, CopartSinkTxKafkaAdapter};
use persister::copart::sink::CopartPersisterSink;
use persister::copart::CopartPersister;
use std::collections::HashMap;
use tokio::join;
use tokio_util::sync::CancellationToken;
use tracing::info;

#[tokio::main]
async fn main() {
    setup_logging("persister");
    let cancellation_token = CancellationToken::new();

    let (sink, sig) = CopartPersisterSink::new(CopartPersister);
    let sink_done = sink.run(cancellation_token.clone());

    let admin = KafkaAdmin::new("localhost:9092");
    admin
        .recreate_topic("copart_response_lot_search")
        .await
        .expect("failed to recreate `copart_response_lot_search` topic");
    admin
        .recreate_topic_with_opts(
            "copart_response_lot_image_blobs",
            &HashMap::from([("max.message.bytes", "100000000")]),
        )
        .await
        .expect("failed to recreate `copart_response_lot_image_blobs` topic");

    let rx_done = KafkaReceiver::new(
        "localhost:9092",
        "consumer_group",
        &[
            "copart_response_lot_search",
            "copart_response_lot_image_blobs",
        ],
    )
    .run_on(
        CopartSinkTxKafkaAdapter {
            cmd_sender: sig.cmd_sender,
        },
        cancellation_token.clone(),
    );
    let tx_done = KafkaSender::new("localhost:9092").run_on(
        CopartSinkRxKafkaAdapter {
            response_receiver: sig.response_receiver,
        },
        cancellation_token.clone(),
    );

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl c event");
    info!("exiting");
    cancellation_token.cancel();
    join!(tx_done.notified(), rx_done.notified(), sink_done.notified());
    info!("exited");
}
