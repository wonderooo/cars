pub mod copart;

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

pub struct Scheduler {
    pub tasks: Vec<Box<dyn Task>>,
}

#[async_trait]
pub trait Task: Send {
    async fn run(&self);
    fn duration(&self) -> tokio::time::Duration;

    fn descriptor(&self) -> Option<&'static str> {
        None
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn with_task(mut self, task: Box<dyn Task>) -> Self {
        self.tasks.push(task);
        self
    }

    pub fn run(self, cancellation_token: CancellationToken) -> Arc<Notify> {
        let handles = self._run_blocking();

        let done = Arc::new(Notify::new());
        tokio::spawn({
            let done = Arc::clone(&done);
            async move {
                cancellation_token.cancelled().await;
                handles.iter().for_each(|handle| handle.abort());
                info!("scheduler closed");
                done.notify_waiters();
            }
        });

        done
    }

    pub async fn run_blocking(self) {
        futures::future::join_all(self._run_blocking()).await;
    }

    fn _run_blocking(self) -> Vec<JoinHandle<()>> {
        self.tasks
            .into_iter()
            .map(|task| {
                debug!("spawning task interval");
                tokio::spawn({
                    async move {
                        let mut interval = tokio::time::interval(task.duration());
                        loop {
                            interval.tick().await;
                            debug!(
                                "running task with descriptor: `{:?}` and duration: `{:?}s`",
                                task.descriptor(),
                                task.duration()
                            );
                            task.run().await;
                        }
                    }
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Scheduler, Task};
    use async_trait::async_trait;

    struct NopTask {
        sender: tokio::sync::mpsc::Sender<()>,
    }

    #[async_trait]
    impl Task for NopTask {
        async fn run(&self) {
            let _ = self.sender.send(()).await;
        }

        fn duration(&self) -> tokio::time::Duration {
            tokio::time::Duration::from_secs(3)
        }
    }

    #[tokio::test(start_paused = true)]
    async fn test_concurrent_task_completion() {
        let (tx1, mut rx1) = tokio::sync::mpsc::channel(1);
        let (tx2, mut rx2) = tokio::sync::mpsc::channel(1);

        let scheduler = Scheduler::new()
            .with_task(Box::new(NopTask { sender: tx1 }))
            .with_task(Box::new(NopTask { sender: tx2 }));
        tokio::spawn(scheduler.run_blocking());

        tokio::time::advance(tokio::time::Duration::from_millis(3000)).await;
        assert_eq!(rx1.recv().await, Some(()));
        assert_eq!(rx2.recv().await, Some(()));
        assert_eq!(rx1.recv().await, Some(()));
        assert_eq!(rx2.recv().await, Some(()));
    }
}
