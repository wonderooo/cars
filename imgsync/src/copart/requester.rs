use async_trait::async_trait;
use common::io::copart::LotImagesVector;
use common::{count_some_none, retry_async};
use futures::StreamExt;
use reqwest::IntoUrl;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio_util::bytes::Bytes;
use tracing::{error, info, instrument};

pub struct LotImageBlobsVector(pub Vec<LotImageBlobs>);

pub struct LotImageBlobs {
    pub standard: Option<Bytes>,
    pub high_res: Option<Bytes>,
    pub thumbnail: Option<Bytes>,
    pub standard_url: Option<String>,
    pub high_res_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub sequence_number: i32,
    pub image_type: String,
}

/// Do not wrap `CopartRequester` in a [`Rc`] or [`Arc`]
/// because [`reqwest::Client`] uses an [`Arc`] internally.
#[derive(Clone)]
pub struct CopartRequester {
    http: reqwest::Client,
    usage_permit: Arc<Semaphore>,
}

#[async_trait]
pub trait CopartRequesterExt {
    async fn download_images(&self, cmds: LotImagesVector) -> LotImageBlobsVector;
}

impl CopartRequester {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            usage_permit: Arc::new(Semaphore::new(32)),
        }
    }

    async fn download_content(&self, url: impl IntoUrl) -> Result<Bytes, reqwest::Error> {
        Ok(self.http.get(url).send().await?.bytes().await?)
    }

    async fn download_content_with_retry(
        &self,
        url: impl IntoUrl + Clone,
        timeout: Duration,
        tries: usize,
    ) -> Result<Bytes, reqwest::Error> {
        retry_async(timeout, tries, || self.download_content(url.to_owned())).await
    }
}

#[async_trait]
impl CopartRequesterExt for CopartRequester {
    #[instrument(skip_all)]
    async fn download_images(&self, cmds: LotImagesVector) -> LotImageBlobsVector {
        let sample_cmds = cmds
            .0
            .iter()
            .flat_map(|li| {
                [
                    li.full_url.to_owned(),
                    li.thumbnail_url.to_owned(),
                    li.full_url.to_owned(),
                ]
            })
            .filter_map(|url| url)
            .take(3)
            .collect::<Vec<String>>();
        info!(sample_lot_images = ?sample_cmds, "sample lot images");

        let option_download_content = async |url: &Option<String>| match url {
            Some(url) => match self
                .download_content_with_retry(url, Duration::from_millis(300), 5)
                .await
            {
                Ok(b) => Some(b),
                Err(e) => {
                    error!(download_error = ?e, "download image blobs failed");
                    None
                }
            },
            None => None,
        };

        let blobs = LotImageBlobsVector(
            futures::stream::iter(cmds.0)
                .map(async |img| {
                    // permits are open per 3 urls, 4 concurrent lot images and 64 semaphore limit
                    // thus maximum socket usage is 3 * 4 * 64 = 768
                    let _permit = unsafe { self.usage_permit.acquire().await.unwrap_unchecked() };
                    let (standard, thumbnail, high_res) = tokio::join!(
                        option_download_content(&img.full_url),
                        option_download_content(&img.thumbnail_url),
                        option_download_content(&img.high_res_url)
                    );
                    drop(_permit);

                    LotImageBlobs {
                        standard,
                        thumbnail,
                        high_res,
                        standard_url: img.full_url,
                        thumbnail_url: img.thumbnail_url,
                        high_res_url: img.high_res_url,
                        sequence_number: img.sequence_number,
                        image_type: img.image_type,
                    }
                })
                .buffer_unordered(4)
                .collect::<Vec<_>>()
                .await,
        );
        info!(blobs = ?blobs, "downloaded blobs");

        blobs
    }
}

impl Debug for LotImageBlobsVector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (some_thumbnail, none_thumbnail) = count_some_none(&self.0, |i| i.thumbnail.as_deref());
        let (some_high, none_high) = count_some_none(&self.0, |i| i.high_res.as_deref());
        let (some_std, none_std) = count_some_none(&self.0, |i| i.standard.as_deref());

        write!(
            f,
            "thumbnail {{some: {some_thumbnail}, none: {none_thumbnail}}}, high_res {{some: {some_high}, none: {none_high}}}, standard {{some: {some_std}, none: {none_std}}}",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::io::copart::LotImages;
    use tokio::time::{Duration, Instant};
    use uuid::Uuid;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn mock_http_server() -> MockServer {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/img"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(20)))
            .mount(&mock_server)
            .await;
        mock_server
    }

    fn random_mock_url(mock_srv: &MockServer) -> String {
        format!(
            "{}/img?_nocache={}",
            mock_srv.uri(),
            Uuid::new_v4().as_simple()
        )
    }

    #[tokio::test]
    async fn test_concurrent_download() {
        let mock_srv = mock_http_server().await;
        let requester = CopartRequester::new();
        let cmd = || LotImages {
            thumbnail_url: Some(random_mock_url(&mock_srv)),
            full_url: Some(random_mock_url(&mock_srv)),
            high_res_url: Some(random_mock_url(&mock_srv)),
            sequence_number: 1,
            image_type: "jpg".to_string(),
        };

        let start_one_cmd = Instant::now();
        requester
            .download_images(LotImagesVector(vec![cmd()]))
            .await;
        let elapsed_one_cmd = start_one_cmd.elapsed();

        let start_three_cmd = Instant::now();
        requester
            .download_images(LotImagesVector(vec![cmd(), cmd(), cmd()]))
            .await;
        let elapsed_three_cmd = start_three_cmd.elapsed();

        assert_eq!(
            elapsed_one_cmd
                .as_millis()
                .abs_diff(elapsed_three_cmd.as_millis())
                < 5,
            true
        );
    }
}
