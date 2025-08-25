use common::logging::setup_logging;
use sched::copart::CopartLotSearchTask;
use sched::Scheduler;
use tokio_util::sync::CancellationToken;
use tracing::info;

#[tokio::main]
async fn main() {
    setup_logging("sched");

    let cancellation_token = CancellationToken::new();
    let sched = Scheduler::new().with_task(Box::new(CopartLotSearchTask::default()));
    let done = sched.run(CancellationToken::clone(&cancellation_token));

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl c event");
    cancellation_token.cancel();
    info!("exiting");
    tokio::join!(done.notified());
    info!("exited");
}
