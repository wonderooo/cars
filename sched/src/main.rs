use common::logging::setup_logging;
use sched::copart::{CopartLoginRefreshTask, CopartLotSearchTask};
use sched::{hours, minutes, ScheduledTask, Scheduler};
use tracing::info;

#[tokio::main]
async fn main() {
    setup_logging("sched");
    info!("starting app");

    Scheduler::run_task(
        ScheduledTask::Interval {
            task: Box::new(CopartLotSearchTask::default()),
            interval: hours(4),
        },
        None,
    );

    Scheduler::run_task(
        ScheduledTask::IntervalDeferred {
            task: Box::new(CopartLoginRefreshTask::default()),
            interval: minutes(30),
        },
        None,
    );

    info!("app started");
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl c event");
    info!("exited");
}
