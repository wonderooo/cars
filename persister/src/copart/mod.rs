pub mod adapter;
pub mod sink;

use async_trait::async_trait;
use aws_sdk_s3::primitives::ByteStream;
use base64::Engine;
use common::bucket::models::NewLotImages;
use common::bucket::MINIO_CLIENT;
use common::io::copart::{Base64Blob, LotNumber};
use common::io::error::GeneralError;
use common::persistence::models::copart::NewLotVehicles;
use common::persistence::schema::lot_vehicle::dsl::lot_vehicle;
use common::persistence::schema::lot_vehicle::lot_number;
use common::persistence::PG_POOL;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use futures::StreamExt;
use tracing::{debug, instrument};

#[async_trait]
pub trait CopartPersisterExt {
    async fn save_new_lot_vehicles(
        &self,
        new_lot_vehicles: NewLotVehicles,
    ) -> Result<Vec<LotNumber>, GeneralError>;

    async fn save_new_lot_images(
        &self,
        new_lot_images: NewLotImages,
    ) -> Result<Vec<LotNumber>, GeneralError>;
}

pub struct CopartPersister;

impl CopartPersister {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CopartPersisterExt for CopartPersister {
    #[instrument(skip_all)]
    async fn save_new_lot_vehicles(
        &self,
        new_lot_vehicles: NewLotVehicles,
    ) -> Result<Vec<LotNumber>, GeneralError> {
        let mut conn = PG_POOL.get().await?;
        conn.transaction::<_, GeneralError, _>(|mut conn| {
            async move {
                debug!(
                    "copart new lot vehicles to save `{}`",
                    new_lot_vehicles.0.len()
                );
                let repeating_lns = lot_vehicle
                    .select(lot_number)
                    .filter(lot_number.eq_any(new_lot_vehicles.0.iter().map(|l| l.lot_number)))
                    .load::<i32>(&mut conn)
                    .await?;
                debug!(
                    "repeating `{}` copart new lot vehicles",
                    repeating_lns.len()
                );
                let unique_lot_vehicles = new_lot_vehicles
                    .0
                    .into_iter()
                    .filter(|lv| !repeating_lns.contains(&lv.lot_number))
                    .collect::<Vec<_>>();
                debug!(
                    "unique `{}` copart new lot vehicles",
                    unique_lot_vehicles.len()
                );
                let k = diesel::insert_into(common::persistence::schema::lot_vehicle::table)
                    .values(&unique_lot_vehicles)
                    .on_conflict_do_nothing() // Discard lot vehicles with already existing lot numbers
                    .execute(&mut conn)
                    .await?;
                debug!("inserted `{k}` copart new lot vehicles");
                Ok(unique_lot_vehicles
                    .into_iter()
                    .map(|lv| lv.lot_number)
                    .collect())
            }
            .scope_boxed()
        })
        .await
    }

    #[instrument(skip_all)]
    async fn save_new_lot_images(
        &self,
        new_lot_images: NewLotImages,
    ) -> Result<Vec<LotNumber>, GeneralError> {
        let put_image = async |key: &String, blob: &Base64Blob, mime_type: &String| {
            MINIO_CLIENT
                .clone()
                .put_object()
                .bucket("lot-images")
                .content_type(mime_type)
                .key(key)
                .body(ByteStream::from(
                    base64::engine::general_purpose::STANDARD
                        .decode(blob)
                        .expect("failed to decode blob"),
                ))
                .send()
                .await
                .expect("failed to put object");
        };

        futures::stream::iter(&new_lot_images.0)
            .for_each_concurrent(16, |img| async move {
                tokio::join!(
                    async {
                        if let Some(ref high_res) = img.high_res {
                            put_image(&high_res.bucket_key, &high_res.blob, &high_res.mime_type)
                                .await
                        }
                    },
                    async {
                        if let Some(ref thumb) = img.thumbnail {
                            put_image(&thumb.bucket_key, &thumb.blob, &thumb.mime_type).await
                        }
                    },
                    async {
                        if let Some(ref std) = img.standard {
                            put_image(&std.bucket_key, &std.blob, &std.mime_type).await
                        }
                    }
                );
            })
            .await;

        let mut conn = PG_POOL.get().await?;
        let new_lot_images: common::persistence::models::copart::NewLotImages =
            new_lot_images.into();
        diesel::insert_into(common::persistence::schema::lot_image::table)
            .values(&new_lot_images.0)
            .execute(&mut conn)
            .await?;

        Ok(vec![])
    }
}
