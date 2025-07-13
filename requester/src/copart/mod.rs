pub mod pool;
pub mod response;
pub mod sink;

use base64::Engine;
use browser::response::lot_images;
use futures::StreamExt;
use reqwest::IntoUrl;
use serde::{Deserialize, Serialize};
use tokio_util::bytes::Bytes;

/// Do not wrap `CopartRequester` in a [`Rc`] or [`Arc`]
/// because [`reqwest::Client`] uses an [`Arc`] internally.
#[derive(Clone)]
pub struct CopartRequester {
    http: reqwest::Client,
}

pub type Base64Blob = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct CopartImageSet {
    pub standard: Option<Base64Blob>,
    pub high_res: Option<Base64Blob>,
    pub thumbnail: Option<Base64Blob>,
}

impl CopartRequester {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }

    pub async fn download_images(&self, response: lot_images::ApiResponse) -> Vec<CopartImageSet> {
        let option_download_content = async |url: Option<String>| {
            if let Some(url) = url {
                return Some(self.download_content(&url).await);
            }
            None
        };

        let n = response.data.images_list.content.len();
        futures::stream::iter(response.data.images_list.content)
            .map(async |img| {
                let (standard, thumbnail, high_res) = tokio::join!(
                    option_download_content(img.full_url),
                    option_download_content(img.thumbnail_url),
                    option_download_content(img.high_res_url)
                );

                CopartImageSet {
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

    async fn download_content(&self, url: impl IntoUrl) -> Bytes {
        self.http
            .get(url)
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap()
    }
}
