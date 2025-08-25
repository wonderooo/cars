use crate::copart::browser::{
    CmdReceiver, CmdSender, CopartBrowser, CopartBrowserCmd, CopartBrowserResponse,
    ResponseReceiver,
};
use crate::copart::error::PoolError;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::Notify;
use tokio::task::AbortHandle;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

pub type PoolResponseReceiver = UnboundedReceiver<CopartBrowserPoolResponse>;
pub type PoolResponseSender = UnboundedSender<CopartBrowserPoolResponse>;

pub struct CopartBrowserPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct CopartBrowserPoolResponse {
    pub inner: CopartBrowserResponse,
    pub ord: usize,
}

impl CopartBrowserPool {
    pub async fn run(
        num_workers: usize,
        proxy_addr: impl Into<SocketAddr> + Clone,
        cancellation_token: CancellationToken,
    ) -> ((CmdSender, PoolResponseReceiver), Arc<Notify>) {
        let (global_cmd_sender, global_cmd_receiver) = tokio::sync::mpsc::unbounded_channel();
        let (global_response_sender, global_response_receiver) =
            tokio::sync::mpsc::unbounded_channel();
        let global_done = Arc::new(Notify::new());

        let (cmd_senders, browsers_done, mut aborts) = Self::spawn_browsers(
            num_workers,
            proxy_addr,
            cancellation_token,
            global_response_sender,
        )
        .await;
        let abort_cmd_receive = Self::cmd_receive_handler(global_cmd_receiver, cmd_senders);

        aborts.push(abort_cmd_receive);
        Self::done_handler(Arc::clone(&global_done), browsers_done, aborts);

        ((global_cmd_sender, global_response_receiver), global_done)
    }

    async fn spawn_browsers(
        num_workers: usize,
        proxy_addr: impl Into<SocketAddr> + Clone,
        cancellation_token: CancellationToken,
        global_response_sender: PoolResponseSender,
    ) -> (VecDeque<CmdSender>, Vec<Arc<Notify>>, Vec<AbortHandle>) {
        futures::stream::iter(0..num_workers)
            .map(async |idx| {
                let ((cmd_sender, response_receiver), done) = CopartBrowser::run(
                    proxy_addr.clone(),
                    CancellationToken::clone(&cancellation_token),
                )
                .await
                .expect("failed to start browser");

                let abort = Self::response_receive_handler(
                    UnboundedSender::clone(&global_response_sender),
                    response_receiver,
                    idx,
                );
                (cmd_sender, done, abort)
            })
            .buffer_unordered(num_workers)
            .collect::<(VecDeque<_>, Vec<_>, Vec<_>)>()
            .await
    }

    fn response_receive_handler(
        global_response_sender: PoolResponseSender,
        mut response_receiver: ResponseReceiver,
        ord: usize,
    ) -> AbortHandle {
        let handle_response = |response: CopartBrowserResponse,
                               global_response_sender: &PoolResponseSender,
                               ord: usize|
         -> Result<(), PoolError> {
            let pool_response = CopartBrowserPoolResponse {
                inner: response,
                ord,
            };
            Ok(global_response_sender.send(pool_response)?)
        };

        let join_handle = tokio::spawn(async move {
            while let Some(response) = response_receiver.recv().await {
                if let Err(e) = handle_response(response, &global_response_sender, ord) {
                    error!("failed to handle response: {}", e);
                }
            }
        });

        join_handle.abort_handle()
    }

    fn cmd_receive_handler(
        mut global_cmd_receiver: CmdReceiver,
        mut local_cmd_senders: VecDeque<CmdSender>,
    ) -> AbortHandle {
        let handle_cmd =
            async |cmd: CopartBrowserCmd, local_cmd_senders: &mut VecDeque<CmdSender>| {
                let sender = local_cmd_senders.pop_front().ok_or(PoolError::NodesEmpty)?;
                sender.send(cmd)?;
                local_cmd_senders.push_back(sender);

                Ok::<(), PoolError>(())
            };

        let join_handle = tokio::spawn(async move {
            while let Some(cmd) = global_cmd_receiver.recv().await {
                if let Err(e) = handle_cmd(cmd, &mut local_cmd_senders).await {
                    error!("failed to handle global cmd receive: {}", e);
                }
            }
        });

        join_handle.abort_handle()
    }

    fn done_handler(
        global_done: Arc<Notify>,
        browsers_done: Vec<Arc<Notify>>,
        aborts: Vec<AbortHandle>,
    ) {
        tokio::spawn({
            async move {
                futures::future::join_all(browsers_done.iter().map(async |n| n.notified().await))
                    .await;
                aborts.iter().for_each(|handle| handle.abort());
                global_done.notify_waiters();
                info!("pool closed")
            }
        });
    }
}
