pub mod copart;

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use tracing::debug;

pub struct Scheduler {
    pub tasks: Vec<ScheduledTask>,
}

#[async_trait]
pub trait Task: Send {
    async fn run(&self, opts: Option<&HashMap<String, String>>);

    fn descriptor(&self) -> Option<&'static str> {
        None
    }
}

pub enum ScheduledTask {
    Interval {
        task: Box<dyn Task>,
        interval: tokio::time::Duration,
    },
    IntervalDeferred {
        task: Box<dyn Task>,
        interval: tokio::time::Duration,
    },
    Timed {
        task: Box<dyn Task>,
        when: chrono::NaiveDateTime,
    },
}

impl Scheduler {
    pub fn run_task(task: ScheduledTask, opts: Option<HashMap<String, String>>) {
        match task {
            ScheduledTask::Interval { task, interval } => {
                Self::spawn_interval_task(task, interval, opts)
            }
            ScheduledTask::IntervalDeferred { task, interval } => {
                Self::spawn_interval_deferred_task(task, interval, opts)
            }
            ScheduledTask::Timed { task, when } => Self::spawn_timed_task(task, when, opts),
        }
    }

    fn spawn_interval_task(
        task: Box<dyn Task>,
        interval: tokio::time::Duration,
        opts: Option<HashMap<String, String>>,
    ) {
        tokio::spawn({
            {
                async move {
                    let mut ticker = tokio::time::interval(interval);
                    loop {
                        ticker.tick().await;
                        debug!(
                            "running task with descriptor: `{:?}` and duration: `{:?}`",
                            task.descriptor(),
                            interval,
                        );
                        task.run(opts.as_ref()).await;
                    }
                }
            }
        });
    }

    fn spawn_interval_deferred_task(
        task: Box<dyn Task>,
        interval: tokio::time::Duration,
        opts: Option<HashMap<String, String>>,
    ) {
        tokio::spawn({
            {
                async move {
                    let mut ticker = tokio::time::interval(interval);
                    ticker.tick().await;
                    loop {
                        ticker.tick().await;
                        debug!(
                            "running task with descriptor: `{:?}` and duration: `{:?}`",
                            task.descriptor(),
                            interval,
                        );
                        task.run(opts.as_ref()).await;
                    }
                }
            }
        });
    }

    fn spawn_timed_task(
        task: Box<dyn Task>,
        when: chrono::NaiveDateTime,
        opts: Option<HashMap<String, String>>,
    ) {
        tokio::spawn(async move {
            let now = Utc::now().naive_utc();
            let delay = (when - now).to_std().unwrap_or(std::time::Duration::ZERO);
            tokio::time::sleep(delay).await;
            debug!(
                "running task with descriptor: `{:?}` set at: `{}`",
                task.descriptor(),
                when,
            );
            task.run(opts.as_ref()).await;
        });
    }
}

pub fn minutes(m: u64) -> tokio::time::Duration {
    tokio::time::Duration::from_secs(m * 60)
}

pub fn hours(h: u64) -> tokio::time::Duration {
    minutes(h * 60)
}

pub fn days(d: u64) -> tokio::time::Duration {
    hours(d * 24)
}

#[cfg(test)]
mod tests {
    use crate::{ScheduledTask, Scheduler, Task};
    use async_trait::async_trait;
    use std::collections::HashMap;

    struct NopTask {
        sender: tokio::sync::mpsc::Sender<()>,
    }

    #[async_trait]
    impl Task for NopTask {
        async fn run(&self, _opts: Option<&HashMap<String, String>>) {
            let _ = self.sender.send(()).await;
        }
    }

    #[tokio::test(start_paused = true)]
    async fn test_concurrent_task_completion() {
        let (tx1, mut rx1) = tokio::sync::mpsc::channel(1);
        let (tx2, mut rx2) = tokio::sync::mpsc::channel(1);

        Scheduler::run_task(
            ScheduledTask::Interval {
                task: Box::new(NopTask { sender: tx1 }),
                interval: tokio::time::Duration::from_secs(3),
            },
            None,
        );

        Scheduler::run_task(
            ScheduledTask::Interval {
                task: Box::new(NopTask { sender: tx2 }),
                interval: tokio::time::Duration::from_secs(3),
            },
            None,
        );

        tokio::time::advance(tokio::time::Duration::from_secs(2)).await;
        assert!(rx1.try_recv().is_err());
        assert!(rx2.try_recv().is_err());

        tokio::time::advance(tokio::time::Duration::from_secs(1)).await;
        assert_eq!(rx1.recv().await, Some(()));
        assert_eq!(rx2.recv().await, Some(()));

        tokio::time::advance(tokio::time::Duration::from_secs(3)).await;
        assert_eq!(rx1.recv().await, Some(()));
        assert_eq!(rx2.recv().await, Some(()));
    }
}
