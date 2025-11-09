use crate::copart::sink::LotImageBlobsResponse;
use async_trait::async_trait;
use aws_sdk_s3::primitives::ByteStream;
use common::bucket::S3_CLIENT;
use common::io::copart::{LotNumber, SyncedImages, SyncedImagesVector};
use common::io::error::GeneralError;
use common::retry_async;
use futures::StreamExt;
use mime_guess::MimeGuess;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio_util::bytes::Bytes;

pub struct CopartUploader {
    usage_permit: Arc<Semaphore>,
}

#[async_trait]
pub trait CopartUploaderExt {
    async fn upload_images(&self, new_lot_images: NewLotImages) -> SyncedImagesVector;
}

#[async_trait]
impl CopartUploaderExt for CopartUploader {
    async fn upload_images(&self, new_lot_images: NewLotImages) -> SyncedImagesVector {
        let synced = futures::stream::iter(new_lot_images.0)
            .map(|img| async move {
                let _permit = unsafe {
                    self.usage_permit
                        .clone()
                        .acquire_owned()
                        .await
                        .unwrap_unchecked()
                };
                let (result_standard, result_thumbnail, result_high_res) = tokio::join!(
                    maybe_upload(img.standard.as_ref()),
                    maybe_upload(img.thumbnail.as_ref()),
                    maybe_upload(img.high_res.as_ref())
                );
                drop(_permit);

                SyncedImages {
                    standard_bucket_key: result_standard.as_ref().map(|m| m.key.to_owned()),
                    standard_mime_type: result_standard.as_ref().map(|m| m.mime_type.to_owned()),
                    standard_source_url: img.standard.map(|i| i.url),
                    thumbnail_bucket_key: result_thumbnail.as_ref().map(|m| m.key.to_owned()),
                    thumbnail_mime_type: result_thumbnail.as_ref().map(|m| m.mime_type.to_owned()),
                    thumbnail_source_url: img.thumbnail.map(|i| i.url),
                    high_res_bucket_key: result_high_res.as_ref().map(|m| m.key.to_owned()),
                    high_res_mime_type: result_high_res.as_ref().map(|m| m.mime_type.to_owned()),
                    high_res_source_url: img.high_res.map(|i| i.url),
                    sequence_number: img.sequence_number,
                    image_type: img.image_type,
                }
            })
            .buffer_unordered(16)
            .collect()
            .await;

        SyncedImagesVector(synced)
    }
}

impl CopartUploader {
    pub fn new() -> Self {
        Self {
            usage_permit: Arc::new(Semaphore::new(32)),
        }
    }
}

async fn maybe_upload(image_info: Option<&ImageInfo>) -> Option<PutObjectMeta> {
    if let Some(image_info) = image_info {
        put_object_with_retry(
            &image_info.bucket_key,
            &image_info.blob,
            &image_info.mime_type,
            Duration::from_millis(300),
            5,
        )
        .await
        .ok()
    } else {
        None
    }
}

struct PutObjectMeta {
    key: String,
    mime_type: String,
}

async fn put_object(
    key: &String,
    content: &Bytes,
    mime_type: &String,
) -> Result<PutObjectMeta, GeneralError> {
    // tokio::util::bytes::Bytes is cheaply cloneable, so we can clone it on every retry
    let stream = ByteStream::from(content.clone());
    Ok(S3_CLIENT
        .put_object()
        .bucket("cars-lot-images")
        .key(key)
        .content_type(mime_type)
        .body(stream)
        .send()
        .await
        .map(|_| PutObjectMeta {
            key: key.to_owned(),
            mime_type: mime_type.to_owned(),
        })
        .map_err(|e| GeneralError::S3(e.to_string()))?)
}

async fn put_object_with_retry(
    key: &String,
    content: &Bytes,
    mime_type: &String,
    timeout: Duration,
    tries: usize,
) -> Result<PutObjectMeta, GeneralError> {
    retry_async(timeout, tries, || put_object(key, content, mime_type)).await
}

pub struct ImageInfo {
    blob: Bytes,
    url: String,
    mime_type: String,
    bucket_key: String,
}

pub struct NewLotImage {
    standard: Option<ImageInfo>,
    thumbnail: Option<ImageInfo>,
    high_res: Option<ImageInfo>,
    sequence_number: i32,
    image_type: String,
}

pub struct NewLotImages(pub Vec<NewLotImage>);

impl ImageInfo {
    fn new(blob: Bytes, url: String, bucket_key: String) -> Self {
        let mime = MimeGuess::from_path(&url).first_or_octet_stream();
        Self {
            blob,
            url,
            bucket_key,
            mime_type: mime.to_string(),
        }
    }
}

fn standard_key(lot_number: LotNumber, sequence_number: i32) -> String {
    format!("{}_{}_standard", lot_number, sequence_number)
}

fn thumbnail_key(lot_number: LotNumber, sequence_number: i32) -> String {
    format!("{}_{}_thumbnail", lot_number, sequence_number)
}

fn high_res_key(lot_number: LotNumber, sequence_number: i32) -> String {
    format!("{}_{}_high-res", lot_number, sequence_number)
}

impl From<LotImageBlobsResponse> for NewLotImages {
    fn from(value: LotImageBlobsResponse) -> Self {
        Self(
            value
                .response
                .0
                .into_iter()
                .map(|i| {
                    let standard = i.standard.map(|std| {
                        ImageInfo::new(
                            std,
                            unsafe { i.standard_url.unwrap_unchecked() },
                            standard_key(value.lot_number, i.sequence_number),
                        )
                    });
                    let thumbnail = i.thumbnail.map(|thumb| {
                        ImageInfo::new(
                            thumb,
                            unsafe { i.thumbnail_url.unwrap_unchecked() },
                            thumbnail_key(value.lot_number, i.sequence_number),
                        )
                    });
                    let high_res = i.high_res.map(|high| {
                        ImageInfo::new(
                            high,
                            unsafe { i.high_res_url.unwrap_unchecked() },
                            high_res_key(value.lot_number, i.sequence_number),
                        )
                    });

                    NewLotImage {
                        standard,
                        thumbnail,
                        high_res,
                        sequence_number: i.sequence_number,
                        image_type: i.image_type,
                    }
                })
                .collect(),
        )
    }
}
