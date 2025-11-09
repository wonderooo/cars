use common::kafka::{KafkaReceiver, KafkaSender};
use common::logging::setup_logging;
use persister::copart::adapter::{CopartSinkRxKafkaAdapter, CopartSinkTxKafkaAdapter};
use persister::copart::sink::CopartPersisterSink;
use persister::copart::CopartPersister;
use tokio_util::sync::CancellationToken;
use tracing::info;

#[cfg(feature = "prof")]
use common::memprof::MemProf;

use common::config::CONFIG;
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
    info!("starting app");
    let cancellation_token = CancellationToken::new();

    let (sink, sig) = CopartPersisterSink::new(CopartPersister);
    let sink_done = sink.run(cancellation_token.clone());

    let rx_done = KafkaReceiver::new(
        CONFIG.kafka.url.to_owned(),
        "consumer_group",
        &[
            "copart_response_lot_search",
            "copart_response_synced_images",
        ],
    )
    .run_on(
        CopartSinkTxKafkaAdapter {
            cmd_sender: sig.cmd_sender,
        },
        cancellation_token.clone(),
    );
    let tx_done = KafkaSender::new(CONFIG.kafka.url.to_owned()).run_on(
        CopartSinkRxKafkaAdapter {
            response_receiver: sig.response_receiver,
        },
        cancellation_token.clone(),
    );

    #[cfg(feature = "prof")]
    let prof_done = MemProf::start("0.0.0.0:6970", cancellation_token.clone());

    info!("app started");
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
