use crate::copart::response::{CopartRequesterPoolResponse, CopartRequesterResponse};
use crate::copart::sink;
use crate::copart::sink::CopartRequesterSink;
use browser::browser::CopartBrowserResponse;
use browser::pool::CopartBrowserPoolResponse;
use futures::StreamExt;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

pub struct ExternalSignaling {
    pub cmd_sender: UnboundedSender<CopartBrowserPoolResponse<CopartBrowserResponse>>,
    pub response_receiver: UnboundedReceiver<CopartRequesterPoolResponse>,
}

pub struct InternalSignaling {
    pub cmd_receiver: UnboundedReceiver<CopartBrowserPoolResponse<CopartBrowserResponse>>,
    pub response_sender: UnboundedSender<CopartRequesterPoolResponse>,
}

pub struct CopartRequesterPool {
    pub sinks: Vec<CopartRequesterSink>,
    pub sink_signals: VecDeque<sink::ExternalSignaling>,
    pub signaling: InternalSignaling,
}

impl CopartRequesterPool {
    pub fn new(n_workers: usize) -> (Self, ExternalSignaling) {
        let (cmd_sender, cmd_receiver) = tokio::sync::mpsc::unbounded_channel();
        let (response_sender, response_receiver) = tokio::sync::mpsc::unbounded_channel();
        let internal_signaling = InternalSignaling {
            cmd_receiver,
            response_sender,
        };
        let external_signaling = ExternalSignaling {
            cmd_sender,
            response_receiver,
        };
        let (sinks, sink_signals) = (0..n_workers)
            .map(|_| CopartRequesterSink::new())
            .collect::<(Vec<_>, VecDeque<_>)>();
        let pool = Self {
            sinks,
            sink_signals,
            signaling: internal_signaling,
        };
        (pool, external_signaling)
    }

    pub fn run(self, cancellation_token: CancellationToken) -> Arc<Notify> {
        let sinks_done = self.run_sinks(cancellation_token.clone());

        let done = Arc::new(Notify::new());
        tokio::spawn({
            let done = done.clone();
            async move {
                cancellation_token.cancelled().await;
                futures::stream::iter(sinks_done)
                    .for_each_concurrent(None, |d| async move {
                        d.notified().await;
                    })
                    .await;
                done.notify_waiters();
            }
        });
        done
    }

    fn run_sinks(self, cancellation_token: CancellationToken) -> Vec<Arc<Notify>> {
        let sinks_done = self
            .sinks
            .into_iter()
            .map(|sink| sink.run(cancellation_token.clone()))
            .collect::<Vec<_>>();

        let (sink_cmd_senders, sink_response_receivers) = self
            .sink_signals
            .into_iter()
            .map(|s| (s.cmd_sender, s.response_receiver))
            .collect::<(VecDeque<_>, Vec<_>)>();
        Self::run_sink_response_receivers(sink_response_receivers, self.signaling.response_sender);
        Self::run_sink_cmd_senders(sink_cmd_senders, self.signaling.cmd_receiver);

        sinks_done
    }

    fn run_sink_response_receivers(
        sink_response_receivers: Vec<UnboundedReceiver<CopartRequesterResponse>>,
        response_sender: UnboundedSender<CopartRequesterPoolResponse>,
    ) {
        sink_response_receivers
            .into_iter()
            .enumerate()
            .for_each(|(n, mut r)| {
                tokio::spawn({
                    let response_sender = UnboundedSender::clone(&response_sender);
                    async move {
                        while let Some(msg) = r.recv().await {
                            response_sender
                                .send(CopartRequesterPoolResponse {
                                    inner: msg,
                                    n_worker: n,
                                })
                                .unwrap();
                        }
                    }
                });
            });
    }

    fn run_sink_cmd_senders(
        mut sink_cmd_senders: VecDeque<
            UnboundedSender<CopartBrowserPoolResponse<CopartBrowserResponse>>,
        >,
        mut cmd_receiver: UnboundedReceiver<CopartBrowserPoolResponse<CopartBrowserResponse>>,
    ) {
        tokio::spawn(async move {
            while let Some(msg) = cmd_receiver.recv().await {
                if let Some(sink_cmd_sender) = sink_cmd_senders.pop_front() {
                    sink_cmd_sender.send(msg).unwrap();
                    sink_cmd_senders.push_back(sink_cmd_sender);
                }
            }
        });
    }
}
