use crate::copart::browser::CmdReceiver;
use chromiumoxide::cdp::browser_protocol::page::NavigateParams;
use chromiumoxide::Page;
use common::io::copart::{AuctionId, CopartCmd, DateTimeRfc3339, LotNumber, LotYear, PageNumber};
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument};

pub struct CmdsHandler {
    navigator: Navigator,
    cmd_receiver: CmdReceiver,
}

pub struct Navigator {
    page: Arc<Page>,
}

impl CmdsHandler {
    pub fn new(page: Arc<Page>, cmd_receiver: CmdReceiver) -> Self {
        Self {
            navigator: Navigator { page },
            cmd_receiver,
        }
    }

    pub async fn handle_blocking(mut self) {
        self.navigator.login().await;

        while let Some(cmd) = self.cmd_receiver.recv().await {
            match cmd {
                CopartCmd::LotSearch {
                    page_number,
                    date_start,
                    date_end,
                    year_start,
                    year_end,
                } => {
                    self.navigator
                        .lot_search(page_number, date_start, date_end, year_start, year_end)
                        .await
                }
                CopartCmd::LoginRefresh => self.navigator.login().await,
                CopartCmd::LotImages(ln) => self.navigator.lot_images(ln).await,
                CopartCmd::Auction(aid) => self.navigator.auction(aid).await,
            }
        }
    }

    pub fn handle(self) -> JoinHandle<()> {
        tokio::spawn(self.handle_blocking())
    }

    pub fn handle_with_cancel(self, cancellation_token: CancellationToken) -> Arc<Notify> {
        let done = Arc::new(Notify::new());
        let join = tokio::spawn(self.handle_blocking());

        tokio::spawn({
            let done = done.clone();
            async move {
                cancellation_token.cancelled().await;
                join.abort();
                info!("cmds handler exited");
                done.notify_waiters();
            }
        });

        done
    }
}

impl Navigator {
    #[instrument(skip(self))]
    pub async fn lot_search(
        &self,
        page_number: PageNumber,
        date_start: DateTimeRfc3339,
        date_end: DateTimeRfc3339,
        year_start: LotYear,
        year_end: LotYear,
    ) {
        let url = format!(
            "https://www.copart.ca/public/lots/search-results?pageNumber={page_number}&dateStart={date_start}&dateEnd={date_end}&yearStart={year_start}&yearEnd={year_end}"
        );
        if let Err(e) = self.goto_and_wait(url).await {
            error!("failed to goto and wait: `{e}`");
        }
    }

    #[instrument(skip(self))]
    pub async fn lot_images(&self, lot_number: LotNumber) {
        let url = format!(
            "https://www.copart.ca/public/data/lotdetails/solr/lotImages/{lot_number}?lotNumber={lot_number}"
        );
        if let Err(e) = self.goto_and_wait(url).await {
            error!("failed to goto and wait: `{e}`");
        }
    }

    #[instrument(skip(self))]
    pub async fn auction(&self, auction_id: AuctionId) {
        let url = format!("https://www.copart.com/auctionDashboard?auctionDetails={auction_id}");
        if let Err(e) = self.goto_and_wait(url).await {
            error!("failed to goto and wait: `{e}`");
        }
    }

    pub async fn login(&self) {
        if let Err(e) = self.goto_and_wait("https://www.copart.ca").await {
            error!("failed to goto and wait: `{e}`");
        }

        // mandatory wait before processing login request - copart needs some time to load JavaScript
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        if let Err(e) = self
            .goto_and_wait("https://www.copart.ca/processLogin")
            .await
        {
            error!("failed to goto and wait: `{e}`");
        }
    }

    pub async fn goto_and_wait(
        &self,
        params: impl Into<NavigateParams>,
    ) -> chromiumoxide::Result<()> {
        self.page.goto(params).await?;
        self.page.wait_for_navigation().await?;
        Ok(())
    }
}
