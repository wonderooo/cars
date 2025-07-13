pub mod copart;

use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub struct Scheduler {
    pub tasks: Vec<Task>,
    pub cancellation_token: CancellationToken,
}

pub struct Task {
    pub duration: tokio::time::Duration,
    pub func: Box<dyn (FnMut() -> Pin<Box<dyn Future<Output = ()> + Send>>) + Send>,
}

impl Scheduler {
    pub fn new(tasks: Vec<Task>, cancellation_token: CancellationToken) -> Self {
        Self {
            tasks,
            cancellation_token,
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn run(self) -> Arc<Notify> {
        let mut aborts = vec![];

        for t in self.tasks {
            let join_handle = tokio::spawn({
                let mut func = t.func;
                let dur = t.duration;

                async move {
                    let mut interval = tokio::time::interval(dur);
                    loop {
                        interval.tick().await;
                        info!("running func");
                        func().await;
                    }
                }
            });
            aborts.push(join_handle.abort_handle());
        }

        let done = Arc::new(Notify::new());
        tokio::spawn({
            let done = Arc::clone(&done);
            async move {
                self.cancellation_token.cancelled().await;
                aborts.iter().for_each(|handle| handle.abort());
                info!("scheduler closed");
                done.notify_waiters();
            }
        });

        done
    }
}

impl Task {
    pub fn new(
        duration: tokio::time::Duration,
        func: Box<dyn (FnMut() -> Pin<Box<dyn Future<Output = ()> + Send>>) + Send>,
    ) -> Self {
        Self { duration, func }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Scheduler, Task};
    use std::time::Duration;
    use tokio_util::sync::CancellationToken;

    #[tokio::test]
    async fn t() {
        let tasks = vec![
            Task::new(
                Duration::from_secs(1),
                Box::new(|| Box::pin(async { println!("Hello, world!") })),
            ),
            Task::new(
                Duration::from_secs(2),
                Box::new(|| Box::pin(async { println!("Hello, world2!") })),
            ),
            Task::new(
                Duration::from_secs(3),
                Box::new(|| Box::pin(async { println!("Hello, world3!") })),
            ),
        ];
        let scheduler = Scheduler::new(tasks, CancellationToken::new());
        scheduler.run();

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
