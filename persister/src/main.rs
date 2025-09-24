use common::kafka::{KafkaAdmin, KafkaReceiver, KafkaSender};
use common::logging::setup_logging;
use persister::copart::adapter::{CopartSinkRxKafkaAdapter, CopartSinkTxKafkaAdapter};
use persister::copart::sink::CopartPersisterSink;
use persister::copart::CopartPersister;
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;
use tracing::info;

#[cfg(feature = "prof")]
use common::memprof::MemProf;

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[allow(non_upper_case_globals)]
#[unsafe(export_name = "malloc_conf")]
#[cfg(feature = "prof")]
pub static malloc_conf: &[u8] = b"prof:true,prof_active:true,lg_prof_sample:19\0";

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
            &HashMap::from([
                ("max.message.bytes", "100000000"),
                ("retention.ms", "1800000"),
            ]),
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

    #[cfg(feature = "prof")]
    let prof_done = MemProf::start("0.0.0.0:6970", cancellation_token.clone());

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl c event");
    info!("exiting");
    cancellation_token.cancel();

    #[cfg(feature = "prof")]
    tokio::join!(
        tx_done.notified(),
        rx_done.notified(),
        sink_done.notified(),
        prof_done.notified()
    );

    #[cfg(not(feature = "prof"))]
    tokio::join!(tx_done.notified(), rx_done.notified(), sink_done.notified(),);

    info!("exited");
}
