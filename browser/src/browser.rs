use crate::error::BrowserError;
use crate::impl_display_and_debug;
use crate::request;
use crate::response::{lot_details, lot_images, lot_search};
use base64::Engine;
use chromiumoxide::cdp::browser_protocol::fetch::{
    ContinueRequestParams, ContinueRequestParamsBuilder, EnableParams, EventRequestPaused,
    GetResponseBodyParams, HeaderEntry, RequestId, RequestPattern, RequestStage,
};
use chromiumoxide::cdp::browser_protocol::network::ResourceType;
use chromiumoxide::{Browser, BrowserConfig, Handler, Page};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use url::Url;

pub type CmdSender = UnboundedSender<CopartBrowserCmd>;
pub type CmdReceiver = UnboundedReceiver<CopartBrowserCmd>;
pub type ResponseReceiver<R> = UnboundedReceiver<R>;
pub type ResponseSender<R> = UnboundedSender<R>;

pub type LotNumber = i32;
pub type PageNumber = usize;
pub type CorrelationId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Structured;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Raw;

pub struct CopartBrowser<R: ResponseGenerator> {
    phantom: PhantomData<R>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CopartBrowserCmd {
    pub correlation_id: CorrelationId,
    pub variant: CopartBrowserCmdVariant,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CopartBrowserCmdVariant {
    LotSearch(PageNumber),
    LotDetails(LotNumber),
    LotImages(LotNumber),
}

impl CopartBrowserCmd {
    fn url(&self) -> String {
        match self.variant {
            CopartBrowserCmdVariant::LotSearch(page_number) => format!(
                "https://www.copart.ca/public/lots/search-results?correlationId={}&pageNumber={page_number}",
                self.correlation_id
            ),
            CopartBrowserCmdVariant::LotDetails(lot_number) => format!(
                "https://www.copart.ca/public/data/lotdetails/solr/{lot_number}?correlationId={}&lotNumber={lot_number}",
                self.correlation_id,
            ),
            CopartBrowserCmdVariant::LotImages(lot_number) => format!(
                "https://www.copart.ca/public/data/lotdetails/solr/lotImages/{lot_number}?correlationId={}&lotNumber={lot_number}",
                self.correlation_id,
            ),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CopartBrowserResponse {
    pub correlation_id: CorrelationId,
    pub variant: CopartBrowserResponseVariant,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LotSearchResponse {
    pub page_number: PageNumber,
    pub response: lot_search::ApiResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LotDetailsResponse {
    pub lot_number: LotNumber,
    pub response: lot_details::ApiResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LotImagesResponse {
    pub lot_number: LotNumber,
    pub response: lot_images::ApiResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CopartBrowserResponseVariant {
    LotSearch(Result<LotSearchResponse, BrowserError>),
    LotDetails(Result<LotDetailsResponse, BrowserError>),
    LotImages(Result<LotImagesResponse, BrowserError>),
}

impl CopartBrowserResponseVariant {
    pub fn topic(&self) -> &str {
        match &self {
            CopartBrowserResponseVariant::LotSearch(_) => "copart_response_lot_search",
            CopartBrowserResponseVariant::LotDetails(_) => unreachable!(),
            CopartBrowserResponseVariant::LotImages(_) => "copart_response_lot_images",
        }
    }
}

impl_display_and_debug!(
    CopartBrowserResponse,
    |s: &CopartBrowserResponse, f: &mut Formatter<'_>| {
        match &s.variant {
            CopartBrowserResponseVariant::LotSearch(r) => {
                write!(f, "{r:?} for correlation id `{}`", s.correlation_id)
            }
            CopartBrowserResponseVariant::LotDetails(_) => write!(f, "LotDetails"),
            CopartBrowserResponseVariant::LotImages(_) => write!(f, "LotImages"),
        }
    }
);

pub trait ResponseGenerator {
    type Response;

    fn create_lot_search_response(
        response_event: &EventRequestPaused,
        page: Arc<Page>,
        correlation_id: CorrelationId,
    ) -> impl Future<Output = Self::Response> + Send;

    fn create_lot_details_response(
        response_event: &EventRequestPaused,
        page: Arc<Page>,
        correlation_id: CorrelationId,
    ) -> impl Future<Output = Self::Response> + Send;

    fn create_lot_images_response(
        response_event: &EventRequestPaused,
        page: Arc<Page>,
        correlation_id: CorrelationId,
    ) -> impl Future<Output = Self::Response> + Send;
}

impl<R> CopartBrowser<R>
where
    R: ResponseGenerator,
    R::Response: Send + 'static,
{
    pub async fn run(
        proxy_addr: impl Into<SocketAddr>,
        cancellation_token: CancellationToken,
    ) -> Result<((CmdSender, ResponseReceiver<R::Response>), Arc<Notify>), BrowserError> {
        let (browser, handler) = Browser::launch(
            BrowserConfig::builder()
                .user_data_dir(PathBuf::from(format!(
                    "/tmp/chromium/{}",
                    uuid::Uuid::new_v4().as_simple()
                )))
                .disable_cache()
                .args(vec![format!("--proxy-server=http://{}", proxy_addr.into())])
                .build()
                .expect("browser failed to build"),
        )
        .await
        .expect("browser failed to launch");

        let (cmd_sender, cmd_receiver) = tokio::sync::mpsc::unbounded_channel();
        let (resp_sender, resp_receiver) = tokio::sync::mpsc::unbounded_channel::<R::Response>();
        let done_notifier = Arc::new(Notify::new());

        tokio::spawn({
            let done_notifier = Arc::clone(&done_notifier);

            let events_join_handle = Self::handle_browser_events(handler);
            let page = Self::setup_page(&browser).await?;
            let user_requests_join_handle =
                Self::handle_browser_user_requests(Arc::clone(&page), cmd_receiver).await?;
            let http_requests_join_handle =
                Self::handle_browser_http_requests(Arc::clone(&page), resp_sender).await?;

            async move {
                tokio::pin!(browser);

                cancellation_token.cancelled().await;
                if let Err(e) = browser.close().await {
                    error!("failed to close browser: {}", e);
                }
                http_requests_join_handle
                    .await
                    .unwrap_or_else(|e| error!("failed to close browser http requests: {}", e));
                events_join_handle
                    .await
                    .unwrap_or_else(|e| error!("failed to close browser events: {}", e));
                user_requests_join_handle.abort();
                info!("browser closed");
                done_notifier.notify_waiters();
            }
        });

        Ok(((cmd_sender, resp_receiver), done_notifier))
    }

    fn handle_browser_events(handler: Handler) -> JoinHandle<()> {
        tokio::spawn(async move {
            tokio::pin!(handler);
            while let Some(h) = handler.next().await {
                if let Err(e) = h {
                    error!("browser handler error: {:?}", e);
                }
            }
        })
    }

    async fn setup_page(browser: &Browser) -> Result<Arc<Page>, BrowserError> {
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
            ]),
            ..Default::default()
        })
        .await?;

        Ok(page)
    }

    async fn handle_browser_user_requests(
        page: Arc<Page>,
        mut cmd_receiver: CmdReceiver,
    ) -> Result<JoinHandle<()>, BrowserError> {
        let join_handle = tokio::spawn(async move {
            while let Some(cmd) = cmd_receiver.recv().await {
                let url = cmd.url();
                if let Err(e) = page.goto(url).await {
                    error!("browser goto error: {}", e);
                }
                if let Err(e) = page.wait_for_navigation().await {
                    error!("browser wait_for_navigation error: {}", e);
                }
            }
        });
        Ok(join_handle)
    }

    async fn handle_browser_http_requests(
        page: Arc<Page>,
        response_sender: ResponseSender<R::Response>,
    ) -> Result<JoinHandle<()>, BrowserError> {
        let mut events = page.event_listener::<EventRequestPaused>().await?;
        let join_handle = tokio::spawn(async move {
            while let Some(event) = events.next().await {
                match event_stage(&event) {
                    RequestStage::Request => match &event.request.url {
                        url if url.contains("/lots/") => {
                            Self::handle_lot_search_request(&page, &event)
                                .await
                                .unwrap_or_else(|e| {
                                    error!("failed to modify browser request: {:?}", e)
                                });
                        }
                        _ => continue,
                    },
                    RequestStage::Response => {
                        Self::branch_http_responses(&event, Arc::clone(&page), &response_sender)
                            .await
                    }
                }
            }
        });

        Ok(join_handle)
    }

    async fn handle_lot_search_request(
        page: impl AsRef<Page>,
        event: impl AsRef<EventRequestPaused>,
    ) -> Result<(), BrowserError> {
        let qp = parse_query_params(&event.as_ref().request.url)?;
        let page_number = qp
            .get("pageNumber")
            .ok_or(BrowserError::PageNumberNotFound)?
            .parse::<usize>()?;
        let sr = request::lot_search::SearchRequest::new(page_number);

        modify_browser_request(
            &page,
            event.as_ref().request_id.clone(),
            "POST",
            serde_json::to_vec(&sr)?,
        )
        .await
    }

    async fn branch_http_responses(
        response_event: &EventRequestPaused,
        page: Arc<Page>,
        response_sender: &ResponseSender<R::Response>,
    ) {
        if response_event
            .request
            .headers
            .inner()
            .get("Cookie")
            .is_none()
        {
            warn!("intercepted preflight response: got no cookies");
            continue_browser_request(&page, response_event.request_id.clone())
                .await
                .unwrap_or_else(|e| error!("continue browser request error: {e}"));
            return;
        }

        let correlation_id = match parse_query_params(&response_event.request.url) {
            Ok(qp) => match qp.get("correlationId") {
                Some(id) => id.clone(),
                None => {
                    warn!(
                        "correlationId not found in response URL: {}",
                        response_event.request.url
                    );
                    continue_browser_request(&page, response_event.request_id.clone())
                        .await
                        .unwrap_or_else(|e| error!("continue browser request error: {e}"));
                    return;
                }
            },
            Err(e) => {
                warn!("failed to parse url query params: {}", e);
                continue_browser_request(&page, response_event.request_id.clone())
                    .await
                    .unwrap_or_else(|e| error!("continue browser request error: {e}"));
                return;
            }
        };

        let user_response = match &response_event.request.url {
            url if url.contains("/lots/") => {
                R::create_lot_search_response(
                    response_event,
                    Arc::clone(&page),
                    correlation_id.to_owned(),
                )
                .await
            }
            url if url.contains("/solr/lotImages/") => {
                R::create_lot_images_response(
                    response_event,
                    Arc::clone(&page),
                    correlation_id.to_owned(),
                )
                .await
            }
            url if url.contains("/solr/") => {
                R::create_lot_details_response(
                    response_event,
                    Arc::clone(&page),
                    correlation_id.to_owned(),
                )
                .await
            }
            url => {
                warn!("intercepted unhandled url: {}", url);
                continue_browser_request(&page, response_event.request_id.clone())
                    .await
                    .unwrap_or_else(|e| error!("continue browser request error: {e}"));
                return;
            }
        };

        response_sender
            .send(user_response)
            .unwrap_or_else(|e| error!("channel send error: {e}"));

        continue_browser_request(&page, response_event.request_id.clone())
            .await
            .unwrap_or_else(|e| error!("continue browser request error: {e}"));
    }
}

impl ResponseGenerator for Structured {
    type Response = CopartBrowserResponse;

    async fn create_lot_search_response(
        response_event: &EventRequestPaused,
        page: Arc<Page>,
        correlation_id: CorrelationId,
    ) -> Self::Response {
        let create_inner = async || {
            let query_params = parse_query_params(&response_event.request.url)?;
            let page_number = query_params
                .get("pageNumber")
                .ok_or(BrowserError::PageNumberNotFound)?
                .parse::<usize>()?;
            let b64 = get_browser_response_body(&page, response_event.request_id.clone()).await?;
            let json = base64_body_into_json(b64)?;
            let response = lot_search::ApiResponse::deserialize(&json)?;

            Ok(LotSearchResponse {
                response,
                page_number,
            })
        };

        CopartBrowserResponse {
            correlation_id,
            variant: CopartBrowserResponseVariant::LotSearch(create_inner().await),
        }
    }

    async fn create_lot_details_response(
        response_event: &EventRequestPaused,
        page: Arc<Page>,
        correlation_id: CorrelationId,
    ) -> Self::Response {
        let create_inner = async || {
            let b64 = get_browser_response_body(&page, response_event.request_id.clone()).await?;
            let json = base64_body_into_json(b64)?;
            let response = lot_details::ApiResponse::deserialize(&json)?;

            Ok(LotDetailsResponse {
                response,
                lot_number: 1,
            })
        };

        CopartBrowserResponse {
            correlation_id,
            variant: CopartBrowserResponseVariant::LotDetails(create_inner().await),
        }
    }

    async fn create_lot_images_response(
        response_event: &EventRequestPaused,
        page: Arc<Page>,
        correlation_id: CorrelationId,
    ) -> Self::Response {
        let create_variant = async || {
            let query_params = parse_query_params(&response_event.request.url)?;
            let lot_number = query_params
                .get("lotNumber")
                .ok_or(BrowserError::PageNumberNotFound)?
                .parse::<i32>()?;
            let b64 = get_browser_response_body(&page, response_event.request_id.clone()).await?;
            let json = base64_body_into_json(b64)?;
            let response = lot_images::ApiResponse::deserialize(&json)?;

            Ok(LotImagesResponse {
                response,
                lot_number,
            })
        };

        CopartBrowserResponse {
            correlation_id,
            variant: CopartBrowserResponseVariant::LotImages(create_variant().await),
        }
    }
}

impl ResponseGenerator for Raw {
    type Response = serde_json::Value;

    async fn create_lot_search_response(
        response_event: &EventRequestPaused,
        page: Arc<Page>,
        _correlation_id: CorrelationId,
    ) -> Self::Response {
        let b64 = get_browser_response_body(&page, response_event.request_id.clone())
            .await
            .unwrap();
        base64_body_into_json(b64).unwrap()
    }

    async fn create_lot_details_response(
        response_event: &EventRequestPaused,
        page: Arc<Page>,
        _correlation_id: CorrelationId,
    ) -> Self::Response {
        let b64 = get_browser_response_body(&page, response_event.request_id.clone())
            .await
            .unwrap();
        base64_body_into_json(b64).unwrap()
    }

    async fn create_lot_images_response(
        response_event: &EventRequestPaused,
        page: Arc<Page>,
        _correlation_id: CorrelationId,
    ) -> Self::Response {
        let b64 = get_browser_response_body(&page, response_event.request_id.clone())
            .await
            .unwrap();
        base64_body_into_json(b64).unwrap()
    }
}

fn event_stage(event: impl AsRef<EventRequestPaused>) -> RequestStage {
    let event = event.as_ref();
    if event.response_headers.is_some() && event.response_status_code.is_some() {
        return RequestStage::Response;
    }
    RequestStage::Request
}

async fn modify_browser_request(
    page: impl AsRef<Page>,
    request_id: RequestId,
    method: impl Into<String>,
    body: impl AsRef<[u8]>,
) -> Result<(), BrowserError> {
    page.as_ref()
        .execute(
            ContinueRequestParamsBuilder::default()
                .request_id(request_id)
                .method(method.into())
                .post_data(base64::engine::general_purpose::STANDARD.encode(body.as_ref()))
                .header(HeaderEntry::new("Content-Type", "application/json"))
                .build()
                .map_err(BrowserError::CdpCommandBuild)?,
        )
        .await?;
    Ok(())
}

async fn continue_browser_request(
    page: impl AsRef<Page>,
    request_id: RequestId,
) -> Result<(), BrowserError> {
    page.as_ref()
        .execute(ContinueRequestParams::new(request_id))
        .await?;
    Ok(())
}

async fn get_browser_response_body(
    page: impl AsRef<Page>,
    request_id: RequestId,
) -> Result<String, BrowserError> {
    Ok(page
        .as_ref()
        .execute(GetResponseBodyParams::new(request_id))
        .await
        .map(|b| b.body.clone())?)
}

fn base64_body_into_json(body: impl AsRef<[u8]>) -> Result<serde_json::Value, BrowserError> {
    let decoded_bytes = base64::engine::general_purpose::STANDARD.decode(body)?;
    let decoded = std::str::from_utf8(&decoded_bytes)?;
    Ok(serde_json::from_str(decoded)?)
}

pub fn parse_query_params(url: impl AsRef<str>) -> Result<HashMap<String, String>, BrowserError> {
    let parsed_url = Url::parse(url.as_ref())?;
    Ok(parsed_url.query_pairs().into_owned().collect())
}
