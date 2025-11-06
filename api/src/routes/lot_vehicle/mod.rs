use crate::domain::LotVehicleWithImages;
use crate::error::{ApiError, ErrorResponse};
use axum::extract::{Path, State};
use axum::Json;
use common::persistence::models::copart::{LotImage, LotVehicle};
use common::persistence::schema::lot_image::sequence_number;
use common::persistence::schema::lot_vehicle::dsl::lot_vehicle;
use common::persistence::schema::lot_vehicle::vin;
use common::persistence::PgPool;
use diesel::{
    BelongingToDsl, ExpressionMethods, GroupedBy, OptionalExtension, QueryDsl, SelectableHelper,
};
use diesel_async::RunQueryDsl;

#[utoipa::path(
    get,
    path = "/lot-vehicle",
    tag = "lot vehicles",
    responses(
        (status = 200, description = "Returns all lot vehicles with images", body = [LotVehicleWithImages])
    )
)]
pub async fn all(State(pool): State<PgPool>) -> Result<Json<Vec<LotVehicleWithImages>>, ApiError> {
    let mut conn = pool.get().await?;
    let all_vehicles = lot_vehicle
        .select(LotVehicle::as_select())
        .limit(500)
        .load(&mut conn)
        .await?;

    let all_images = LotImage::belonging_to(&all_vehicles)
        .select(LotImage::as_select())
        .load(&mut conn)
        .await?;

    let lot_vehicles_with_images = all_images
        .grouped_by(&all_vehicles)
        .into_iter()
        .zip(all_vehicles)
        .map(|(images, vehicle)| LotVehicleWithImages {
            lot_vehicle: vehicle.into(),
            lot_images: images.into_iter().map(|x| x.into()).collect(),
        })
        .collect::<Vec<LotVehicleWithImages>>();

    Ok(Json(lot_vehicles_with_images))
}

#[utoipa::path(
    get,
    path = "/lot-vehicle/{ln}",
    tag = "lot vehicle by lot number",
    params(
        ("ln" = i32, Path, description = "The lot number of the vehicle")
    ),
    responses(
        (status = 200, description = "Returns a lot vehicle with images", body = LotVehicleWithImages),
        (status = 404, description = "Returns a error when lot number does not exist", body = ErrorResponse)
    )
)]
pub async fn by_ln(
    Path(ln): Path<i32>,
    State(pool): State<PgPool>,
) -> Result<Json<LotVehicleWithImages>, ApiError> {
    let mut conn = pool.get().await?;
    let vehicle = lot_vehicle
        .find(&ln)
        .select(LotVehicle::as_select())
        .first(&mut conn)
        .await
        .optional()?
        .ok_or(ApiError::LotVehicleNotFoundLn(ln))?;

    let all_images = LotImage::belonging_to(&vehicle)
        .order(sequence_number.asc())
        .select(LotImage::as_select())
        .load(&mut conn)
        .await?;

    let lot_vehicle_with_images = LotVehicleWithImages {
        lot_vehicle: vehicle.into(),
        lot_images: all_images.into_iter().map(|x| x.into()).collect(),
    };
    Ok(Json(lot_vehicle_with_images))
}

#[utoipa::path(
    get,
    path = "/lot-vehicle/vin/{vin}",
    tag = "lot vehicle by vin number",
    params(
        ("vin" = String, Path, description = "The vin number of the vehicle")
    ),
    responses(
        (status = 200, description = "Returns a lot vehicle with images", body = LotVehicleWithImages),
        (status = 404, description = "Returns a error when lot number does not exist", body = ErrorResponse)
    )
)]
pub async fn by_vin(
    Path(v): Path<String>,
    State(pool): State<PgPool>,
) -> Result<Json<LotVehicleWithImages>, ApiError> {
    let mut conn = pool.get().await?;
    let vehicle = lot_vehicle
        .filter(vin.eq(&v))
        .select(LotVehicle::as_select())
        .first(&mut conn)
        .await
        .optional()?
        .ok_or(ApiError::LotVehicleNotFoundVin(v))?;

    let all_images = LotImage::belonging_to(&vehicle)
        .order(sequence_number.asc())
        .select(LotImage::as_select())
        .load(&mut conn)
        .await?;

    let lot_vehicle_with_images = LotVehicleWithImages {
        lot_vehicle: vehicle.into(),
        lot_images: all_images.into_iter().map(|x| x.into()).collect(),
    };
    Ok(Json(lot_vehicle_with_images))
}
