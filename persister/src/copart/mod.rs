pub mod adapter;
pub mod sink;

use crate::orm::models::copart::{NewLotImages, NewLotVehicles};
use crate::orm::schema::lot_vehicle::dsl::lot_vehicle;
use crate::orm::schema::lot_vehicle::lot_number;
use crate::orm::PG_POOL;
use async_trait::async_trait;
use common::io::copart::LotNumber;
use common::io::error::GeneralError;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
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
                let k = diesel::insert_into(crate::orm::schema::lot_vehicle::table)
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
        let mut conn = PG_POOL.get().await?;

        diesel::insert_into(crate::orm::schema::lot_image::table)
            .values(&new_lot_images.0)
            .execute(&mut conn)
            .await?;

        let lns = new_lot_images
            .0
            .into_iter()
            .map(|i| i.lot_vehicle_number)
            .collect::<Vec<_>>();
        Ok(lns)
    }
}
