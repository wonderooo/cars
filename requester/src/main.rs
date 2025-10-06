use common::kafka::{KafkaReceiver, KafkaSender};
use common::logging::setup_logging;
use requester::copart::adapter::{CopartSinkRxKafkaAdapter, CopartSinkTxKafkaAdapter};
use requester::copart::client::CopartRequester;
use requester::copart::sink::CopartRequesterSink;
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

    #[cfg(feature = "prof")]
    let prof_done = MemProf::start("0.0.0.0:6969", cancellation_token.clone());

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl c event");
    info!("exiting");
    cancellation_token.cancel();

    #[cfg(feature = "prof")]
    tokio::join!(
        tx_done.notified(),
        rx_done.notified(),
        copart_sink_done.notified(),
        prof_done.notified(),
    );

    #[cfg(not(feature = "prof"))]
    tokio::join!(
        tx_done.notified(),
        rx_done.notified(),
        copart_sink_done.notified(),
    );
    info!("exited");
}
