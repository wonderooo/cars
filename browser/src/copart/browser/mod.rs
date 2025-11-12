use crate::copart::browser::handlers::cmd::CmdsHandler;
use crate::copart::browser::handlers::engine::BrowserEngineHandler;
use crate::copart::browser::handlers::http::HttpHandler;
use crate::copart::browser::handlers::ws::WsHandler;
use chromiumoxide::cdp::browser_protocol::fetch::{EnableParams, RequestPattern, RequestStage};
use chromiumoxide::cdp::browser_protocol::network::ResourceType;
use chromiumoxide::{Browser, BrowserConfig, Handler, Page};
use common::io::copart::{CopartCmd, CopartResponse};
use common::io::error::GeneralError;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::debug;

pub mod handlers;
pub mod smf;

pub type CmdSender = Sender<CopartCmd>;
pub type CmdReceiver = Receiver<CopartCmd>;
pub type ResponseReceiver = Receiver<CopartResponse>;
pub type ResponseSender = Sender<CopartResponse>;

pub struct CopartBrowser;

impl CopartBrowser {
    pub async fn run(
        host: Option<String>,
        port: Option<u16>,
        cancellation_token: CancellationToken,
    ) -> Result<((CmdSender, ResponseReceiver), Arc<Notify>), GeneralError> {
        debug!("running browser on proxy: {:?}:{:?}", host, port);
        let mut args = vec![
            "--no-sandbox".to_string(),
            "--disable-dev-shm-usage".to_string(),
        ];
        if let (Some(host), Some(port)) = (host, port) {
            args.push(format!("--proxy-server=http://{host}:{port}"));
        }

        let (browser, handler) = Browser::launch(
            BrowserConfig::builder()
                .user_data_dir(PathBuf::from(format!(
                    "/tmp/chromium/{}",
                    uuid::Uuid::new_v4().as_simple()
                )))
                .disable_cache()
                .args(args)
                .build()
                .expect("browser failed to build"),
        )
        .await
        .expect("browser failed to launch");

        let (cmd_sender, cmd_receiver) = tokio::sync::mpsc::channel(32);
        let (resp_sender, resp_receiver) = tokio::sync::mpsc::channel(32);
        let done = Self::run_and_forget_handlers(
            handler,
            browser,
            cmd_receiver,
            resp_sender,
            &cancellation_token,
        )
        .await;

        Ok(((cmd_sender, resp_receiver), done))
    }

    async fn run_and_forget_handlers(
        handler: Handler,
        mut browser: Browser,
        cmd_receiver: CmdReceiver,
        resp_sender: ResponseSender,
        cancellation_token: &CancellationToken,
    ) -> Arc<Notify> {
        let engine_task = BrowserEngineHandler::new(handler).handle();
        // setup_page must be called after the browser engine handler is started
        let page = Self::setup_page(&browser)
            .await
            .expect("failed to setup page");

        let cmds_task = CmdsHandler::new(page.clone(), cmd_receiver).handle();
        let http_task = HttpHandler::new(page.clone(), resp_sender.clone()).handle();
        let ws_task = WsHandler::new(page.clone(), resp_sender.clone()).handle();

        let done = Arc::new(Notify::new());
        tokio::spawn({
            let cancellation_token = cancellation_token.clone();
            let done = done.clone();
            async move {
                cancellation_token.cancelled().await;
                browser.close().await.expect("failed to close browser");
                engine_task
                    .await
                    .expect("failed to await browser engine handler");
                let _ = tokio::join!(cmds_task, http_task, ws_task);
                done.notify_waiters();
            }
        });

        done
    }

    async fn setup_page(browser: &Browser) -> Result<Arc<Page>, GeneralError> {
        let page = Arc::new(browser.new_page("about:blank").await?);
        page.enable_stealth_mode().await?;
        page.execute(EnableParams {
            patterns: Some(vec![
                // lot-search response
                RequestPattern {
                    url_pattern: Some("*/lots/*".to_string()),
                    resource_type: Some(ResourceType::Document),
                    request_stage: Some(RequestStage::Response),
                },
                // lot-details response
                RequestPattern {
                    url_pattern: Some("*/solr/*".to_string()),
                    resource_type: Some(ResourceType::Document),
                    request_stage: Some(RequestStage::Response),
                },
                // lot-images response
                RequestPattern {
                    url_pattern: Some("*/solr/lotImages/*".to_string()),
                    resource_type: Some(ResourceType::Document),
                    request_stage: Some(RequestStage::Response),
                },
                // lot-search request (for GET to POST transmute)
                RequestPattern {
                    url_pattern: Some("*/lots/*".to_string()),
                    resource_type: Some(ResourceType::Document),
                    request_stage: Some(RequestStage::Request),
                },
                // auction request (for GET to POST transmute)
                RequestPattern {
                    url_pattern: Some("*/processLogin".to_string()),
                    resource_type: Some(ResourceType::Document),
                    request_stage: Some(RequestStage::Request),
                },
            ]),
            ..Default::default()
        })
        .await?;

        Ok(page)
    }
}
