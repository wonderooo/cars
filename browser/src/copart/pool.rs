use crate::copart::browser::{
    CmdReceiver, CmdSender, CopartBrowser, ResponseReceiver, ResponseSender,
};
use common::io::copart::{CopartCmd, CopartResponse};
use common::io::error::GeneralError;
use futures::StreamExt;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::task::AbortHandle;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

pub struct CopartBrowserPool {
    host: String,
    port: u16,
    cancellation_token: CancellationToken,

    global_response_sender: ResponseSender,
    global_cmd_receiver: CmdReceiver,
}

pub struct ExternalSignaling {
    pub cmd_sender: CmdSender,
    pub response_receiver: ResponseReceiver,
}

impl CopartBrowserPool {
    pub fn new(
        host: String,
        port: u16,
        cancellation_token: CancellationToken,
    ) -> (Self, ExternalSignaling) {
        let (global_cmd_sender, global_cmd_receiver) = tokio::sync::mpsc::channel(32);
        let (global_response_sender, global_response_receiver) = tokio::sync::mpsc::channel(32);

        let pool = Self {
            host,
            port,
            cancellation_token,
            global_response_sender,
            global_cmd_receiver,
        };
        let external_signaling = ExternalSignaling {
            cmd_sender: global_cmd_sender,
            response_receiver: global_response_receiver,
        };
        (pool, external_signaling)
    }

    pub async fn run(self, num_workers: usize) -> Arc<Notify> {
        let global_done = Arc::new(Notify::new());

        let (cmd_senders, browsers_done, mut aborts) = self.spawn_browsers(num_workers).await;

        let abort_cmd_receive = self.cmd_receive_handler(cmd_senders);
        aborts.push(abort_cmd_receive);

        Self::done_handler(Arc::clone(&global_done), browsers_done, aborts);
        global_done
    }

    async fn spawn_browser(&self) -> (CmdSender, Arc<Notify>, AbortHandle) {
        let ((cmd_sender, response_receiver), done) =
            CopartBrowser::run(None, None, self.cancellation_token.clone())
                .await
                .expect("failed to start browser");

        let abort =
            Self::response_receive_handler(self.global_response_sender.clone(), response_receiver);
        (cmd_sender, done, abort)
    }

    async fn spawn_proxied_browser(&self) -> (CmdSender, Arc<Notify>, AbortHandle) {
        let ((cmd_sender, response_receiver), done) = CopartBrowser::run(
            Some(self.host.clone()),
            Some(self.port),
            self.cancellation_token.clone(),
        )
        .await
        .expect("failed to start browser");

        let abort =
            Self::response_receive_handler(self.global_response_sender.clone(), response_receiver);
        (cmd_sender, done, abort)
    }

    async fn spawn_browsers(
        &self,
        num_workers: usize,
    ) -> (VecDeque<CmdSender>, Vec<Arc<Notify>>, Vec<AbortHandle>) {
        futures::stream::iter(0..num_workers)
            .map(async |_| self.spawn_proxied_browser().await)
            .buffer_unordered(num_workers)
            .collect::<(VecDeque<_>, Vec<_>, Vec<_>)>()
            .await
    }

    fn response_receive_handler(
        global_response_sender: ResponseSender,
        mut response_receiver: ResponseReceiver,
    ) -> AbortHandle {
        let handle_response = async |response: CopartResponse,
                                     global_response_sender: &ResponseSender|
               -> Result<(), GeneralError> {
            Ok(global_response_sender.send(response).await?)
        };

        let join_handle = tokio::spawn(async move {
            while let Some(response) = response_receiver.recv().await {
                if let Err(e) = handle_response(response, &global_response_sender).await {
                    error!("failed to handle response: {}", e);
                }
            }
        });

        join_handle.abort_handle()
    }

    fn cmd_receive_handler(mut self, mut local_cmd_senders: VecDeque<CmdSender>) -> AbortHandle {
        let handle_cmd = async |cmd: CopartCmd, local_cmd_senders: &mut VecDeque<CmdSender>| {
            let sender = local_cmd_senders
                .pop_front()
                .ok_or(GeneralError::BrowserPoolEmpty)?;
            sender.send(cmd).await?;
            local_cmd_senders.push_back(sender);

            Ok::<(), GeneralError>(())
        };

        let join_handle = tokio::spawn(async move {
            while let Some(cmd) = self.global_cmd_receiver.recv().await {
                println!("cmd: {:?}", cmd);
                match cmd {
                    CopartCmd::Auction(_) => {
                        let (cmd_sender, _, _) = self.spawn_browser().await;
                        if let Err(e) = cmd_sender.send(cmd).await {
                            error!("failed to handle global cmd receive: {}", e);
                        }
                    }
                    CopartCmd::LoginRefresh => {
                        for sender in &local_cmd_senders {
                            if let Err(e) = sender.send(CopartCmd::LoginRefresh).await {
                                error!("failed to send cmd to local sender: {e}");
                            }
                        }
                    }
                    CopartCmd::LotSearch { .. } | CopartCmd::LotImages(_) => {
                        if let Err(e) = handle_cmd(cmd, &mut local_cmd_senders).await {
                            error!("failed to handle global cmd receive: {}", e);
                        }
                    }
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
