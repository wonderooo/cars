use crate::copart::io::{CopartImageBlobCmd, CopartImageSet};
use base64::Engine;
use futures::StreamExt;
use reqwest::IntoUrl;
use tokio_util::bytes::Bytes;

/// Do not wrap `CopartRequester` in a [`Rc`] or [`Arc`]
/// because [`reqwest::Client`] uses an [`Arc`] internally.
#[derive(Clone)]
pub struct CopartRequester {
    http: reqwest::Client,
}

pub trait ICopartRequester {
    fn download_images(
        &self,
        cmds: Vec<CopartImageBlobCmd>,
    ) -> impl Future<Output = Vec<CopartImageSet>> + Send;
}

impl CopartRequester {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
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

impl ICopartRequester for CopartRequester {
    async fn download_images(&self, cmds: Vec<CopartImageBlobCmd>) -> Vec<CopartImageSet> {
        let option_download_content = async |url: Option<String>| {
            if let Some(url) = url {
                return Some(self.download_content(&url).await);
            }
            None
        };

        let n = cmds.len();
        futures::stream::iter(cmds)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::copart::sink::{CopartRequesterSink, MsgIn};
    use std::time::Duration;

    struct NopCopartRequester;

    impl ICopartRequester for NopCopartRequester {
        async fn download_images(&self, _cmds: Vec<CopartImageBlobCmd>) -> Vec<CopartImageSet> {
            tokio::time::sleep(Duration::from_millis(100)).await;
            vec![CopartImageSet {
                standard: None,
                high_res: None,
                thumbnail: None,
            }]
        }
    }

    #[tokio::test]
    async fn t1() {
        let (sink, mut sig) = CopartRequesterSink::new(NopCopartRequester);
        tokio::spawn(sink.run_blocking());
        sig.cmd_sender
            .send(MsgIn::LotImageBlobs { cmds: vec![] })
            .unwrap();
        let l = sig.response_receiver.recv().await;
        dbg!("{:?}", l);
    }
}
