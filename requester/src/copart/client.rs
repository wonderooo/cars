use async_trait::async_trait;
use base64::Engine;
use common::io::copart::{LotImageBlobs, LotImages};
use futures::StreamExt;
use reqwest::IntoUrl;
use tokio_util::bytes::Bytes;

/// Do not wrap `CopartRequester` in a [`Rc`] or [`Arc`]
/// because [`reqwest::Client`] uses an [`Arc`] internally.
#[derive(Clone)]
pub struct CopartRequester {
    http: reqwest::Client,
}

#[async_trait]
pub trait CopartRequesterExt {
    async fn download_images(&self, cmds: Vec<LotImages>) -> Vec<LotImageBlobs>;
}

impl CopartRequester {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }

    async fn download_content(&self, url: impl IntoUrl) -> Result<Bytes, reqwest::Error> {
        Ok(self.http.get(url).send().await?.bytes().await?)
    }
}

#[async_trait]
impl CopartRequesterExt for CopartRequester {
    async fn download_images(&self, cmds: Vec<LotImages>) -> Vec<LotImageBlobs> {
        let option_download_content = async |url: Option<String>| match url {
            Some(url) => self.download_content(&url).await.ok(),
            None => None,
        };

        let n = cmds.len();
        futures::stream::iter(cmds)
            .map(async |img| {
                let (standard, thumbnail, high_res) = tokio::join!(
                    option_download_content(img.full_url),
                    option_download_content(img.thumbnail_url),
                    option_download_content(img.high_res_url)
                );

                LotImageBlobs {
                    standard: standard
                        .map(|bytes| base64::engine::general_purpose::STANDARD.encode(bytes)),
                    thumbnail: thumbnail
                        .map(|bytes| base64::engine::general_purpose::STANDARD.encode(bytes)),
                    high_res: high_res
                        .map(|bytes| base64::engine::general_purpose::STANDARD.encode(bytes)),
                }
            })
            .buffer_unordered(n)
            .collect::<Vec<_>>()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        };

        let start_one_cmd = Instant::now();
        requester.download_images(vec![cmd()]).await;
        let elapsed_one_cmd = start_one_cmd.elapsed();

        let start_three_cmd = Instant::now();
        requester.download_images(vec![cmd(), cmd(), cmd()]).await;
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
