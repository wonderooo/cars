use crate::copart::browser::ResponseSender;
use crate::copart::{request, response};
use base64::Engine;
use chromiumoxide::cdp::browser_protocol::fetch::{
    ContinueRequestParams, ContinueRequestParamsBuilder, EventRequestPaused, GetResponseBodyParams,
    HeaderEntry, RequestId, RequestStage,
};
use chromiumoxide::Page;
use common::io::copart::{
    CopartResponse, DateTimeRfc3339, LotImagesResponse, LotNumber, LotSearchResponse, LotYear,
    PageNumber,
};
use common::io::error::GeneralError;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument};
use url::Url;

pub struct HttpHandler {
    page: Arc<Page>,
    request_handler: RequestHandler,
    response_handler: ResponseHandler,
}

/// `RequestHandler` is responsible for handling web traffic before it leaves the browser.
/// Its handling is strictly coupled with `chromiumoxide::RequestPattern` set up by the browser
/// and `Navigator` in `CmdsHandler`.
struct RequestHandler {
    page: Arc<Page>,
}

/// `ResponseHandler` is responsible for handling web traffic after the browser receives it.
/// Its handling is strictly coupled with `chromiumoxide::RequestPattern` set up by the browser
/// and `Navigator` in `CmdsHandler`.
struct ResponseHandler {
    page: Arc<Page>,
    response_sender: ResponseSender,
}

impl HttpHandler {
    pub fn new(page: Arc<Page>, response_sender: ResponseSender) -> Self {
        Self {
            page: page.clone(),
            request_handler: RequestHandler { page: page.clone() },
            response_handler: ResponseHandler {
                page: page.clone(),
                response_sender,
            },
        }
    }

    pub async fn handle_blocking(self) {
        let mut events = self
            .page
            .event_listener::<EventRequestPaused>()
            .await
            .expect("failed to get event listener");

        while let Some(event) = events.next().await {
            match event_stage(&event) {
                RequestStage::Request => self.request_handler.handle(event).await,
                RequestStage::Response => self.response_handler.handle(event).await,
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
                info!("http handler exited");
                done.notify_waiters();
            }
        });

        done
    }
}

impl RequestHandler {
    #[instrument(skip_all)]
    async fn handle(&self, event: Arc<EventRequestPaused>) {
        println!("request: {:?}", event.request.url);
        let request_id = event.request_id.clone();
        match &event.request.url {
            // this pattern depends on lot_search in CmdsHandler and browser's request pattern
            url if url.contains("/lots/") => {
                if let Err(e) = self.modify_lot_search(event).await {
                    // if request fails to modify then it is not continued
                    error!("failed to modify lot search request: `{e}`");
                }
            }
            // this pattern depends on login in CmdsHandler and browser's request pattern
            url if url.contains("/processLogin") => {
                if let Err(e) = self.modify_login(event).await {
                    // if request fails to modify then it is not continued
                    error!("failed to modify login request: `{e}`");
                }
            }
            _ => continue_browser_request(&self.page, event.request_id.clone())
                .await
                .unwrap_or_else(|e| error!("continue unmatched browser request error: `{e}`")),
        }
        continue_browser_request(&self.page, request_id)
            .await
            .unwrap_or_else(|e| error!("continue unmatched browser request error: `{e}`"))
    }

    async fn modify_lot_search(&self, event: Arc<EventRequestPaused>) -> Result<(), GeneralError> {
        let query_params = query_params(&event.request.url)?;
        let page_number = query_params
            .get("pageNumber")
            .ok_or(GeneralError::PageNumberNotFound)?
            .parse::<PageNumber>()?;
        let date_start: &DateTimeRfc3339 = query_params
            .get("dateStart")
            .ok_or(GeneralError::PageNumberNotFound)?;
        let date_end: &DateTimeRfc3339 = query_params
            .get("dateEnd")
            .ok_or(GeneralError::PageNumberNotFound)?;
        let year_start = query_params
            .get("yearStart")
            .ok_or(GeneralError::PageNumberNotFound)?
            .parse::<LotYear>()?;
        let year_end = query_params
            .get("yearEnd")
            .ok_or(GeneralError::PageNumberNotFound)?
            .parse::<LotYear>()?;

        let request_body = request::lot_search::SearchRequest::new(page_number)
            .with_year(&year_start, &year_end)
            .with_auction_date(&date_start, &date_end);
        let request_body_bytes = serde_json::to_vec(&request_body)?;
        self.modify_request_to_post_and_continue(event.request_id.clone(), request_body_bytes)
            .await?;
        Ok(())
    }

    async fn modify_login(&self, event: Arc<EventRequestPaused>) -> Result<(), GeneralError> {
        let request_body = request::login::LoginRequest::new();
        let request_body_bytes = serde_json::to_vec(&request_body)?;
        self.modify_request_to_post_and_continue(event.request_id.clone(), request_body_bytes)
            .await?;
        Ok(())
    }

    async fn modify_request_to_post_and_continue(
        &self,
        request_id: RequestId,
        body: impl AsRef<[u8]>,
    ) -> Result<(), GeneralError> {
        self.page
            .execute(
                ContinueRequestParamsBuilder::default()
                    .request_id(request_id)
                    .method("POST")
                    .post_data(base64::engine::general_purpose::STANDARD.encode(body.as_ref()))
                    .header(HeaderEntry::new("Content-Type", "application/json"))
                    .build()
                    .map_err(GeneralError::CdpCommandBuild)?,
            )
            .await?;
        Ok(())
    }
}

impl ResponseHandler {
    async fn handle(&self, event: Arc<EventRequestPaused>) {
        let request_id = event.request_id.clone();
        match &event.request.url {
            url if url.contains("/lots/") => {
                if let Err(e) = self.process_lot_search(event).await {
                    error!("failed to process lot search response: `{e}`");
                }
            }
            url if url.contains("/solr/lotImages") => {
                if let Err(e) = self.process_lot_images(event).await {
                    error!("failed to process lot images response: `{e}`");
                }
            }
            _ => {}
        }

        continue_browser_request(&self.page, request_id)
            .await
            .unwrap_or_else(|e| error!("continue unmatched browser request error: `{e}`"));
    }

    async fn process_lot_search(&self, event: Arc<EventRequestPaused>) -> Result<(), GeneralError> {
        let maybe_response = self.create_lot_search(event).await;
        self.response_sender
            .send(CopartResponse::LotSearch(maybe_response))
            .await?;
        Ok(())
    }

    async fn create_lot_search(
        &self,
        event: Arc<EventRequestPaused>,
    ) -> Result<LotSearchResponse, GeneralError> {
        let query_params = query_params(&event.request.url)?;
        let page_number = query_params
            .get("pageNumber")
            .ok_or(GeneralError::PageNumberNotFound)?
            .parse::<PageNumber>()?;

        let base64_body = self
            .get_browser_response_body(event.request_id.clone())
            .await?;
        let body = base64::engine::general_purpose::STANDARD.decode(&base64_body)?;
        let unmarshalled = serde_json::from_slice::<response::lot_search::ApiResponse>(&body)?;
        let response = LotSearchResponse {
            response: unmarshalled.into(),
            page_number,
        };
        Ok(response)
    }

    async fn process_lot_images(&self, event: Arc<EventRequestPaused>) -> Result<(), GeneralError> {
        let maybe_response = self.create_lot_images(event).await;
        self.response_sender
            .send(CopartResponse::LotImages(maybe_response))
            .await?;
        Ok(())
    }

    async fn create_lot_images(
        &self,
        event: Arc<EventRequestPaused>,
    ) -> Result<LotImagesResponse, GeneralError> {
        let query_params = query_params(&event.request.url)?;
        let lot_number = query_params
            .get("lotNumber")
            .ok_or(GeneralError::LotNumberNotFound)?
            .parse::<LotNumber>()?;

        let base64_body = self
            .get_browser_response_body(event.request_id.clone())
            .await?;
        let body = base64::engine::general_purpose::STANDARD.decode(&base64_body)?;
        let unmarshalled = serde_json::from_slice::<response::lot_images::ApiResponse>(&body)?;
        let response = LotImagesResponse {
            response: unmarshalled.into(),
            lot_number,
        };
        Ok(response)
    }

    async fn get_browser_response_body(
        &self,
        request_id: RequestId,
    ) -> Result<String, GeneralError> {
        Ok(self
            .page
            .execute(GetResponseBodyParams::new(request_id))
            .await
            .map(|b| b.body.clone())?)
    }
}

fn event_stage(event: impl AsRef<EventRequestPaused>) -> RequestStage {
    let event = event.as_ref();
    if event.response_headers.is_some() && event.response_status_code.is_some() {
        return RequestStage::Response;
    }
    RequestStage::Request
}

async fn continue_browser_request(
    page: impl AsRef<Page>,
    request_id: RequestId,
) -> Result<(), GeneralError> {
    page.as_ref()
        .execute(ContinueRequestParams::new(request_id))
        .await?;
    Ok(())
}

fn query_params(url: impl AsRef<str>) -> Result<HashMap<String, String>, GeneralError> {
    let parsed_url = Url::parse(url.as_ref())?;
    Ok(parsed_url.query_pairs().into_owned().collect())
}
