use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LotVehicleWithImages {
    #[serde(flatten)]
    pub lot_vehicle: LotVehicle,
    pub lot_images: Vec<LotImage>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LotVehicle {
    pub lot_number: i32,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub vehicle_type: String,
    pub vin: Option<String>,
    pub estimated_retail_value: f64,
    pub estimated_repair_cost: f64,
    pub odometer: f64,
    pub odometer_status: Option<String>,
    pub engine_name: Option<String>,
    pub engine_cylinders: Option<String>,
    pub currency: String,
    #[schema(value_type = Option<String>, example = "2025-10-13T15:30:00")]
    pub sale_date: Option<chrono::NaiveDateTime>,
    pub main_damage: String,
    pub other_damage: Option<String>,
    pub country: String,
    pub state: String,
    pub transmission: Option<String>,
    pub color: String,
    pub fuel_type: Option<String>,
    pub drive_type: Option<String>,
    pub keys_status: Option<String>,
}

impl From<common::persistence::models::copart::LotVehicle> for LotVehicle {
    fn from(value: common::persistence::models::copart::LotVehicle) -> Self {
        Self {
            lot_number: value.lot_number,
            make: value.make,
            model: value.model,
            year: value.year,
            vehicle_type: value.vehicle_type,
            vin: value.vin,
            estimated_retail_value: value.estimated_retail_value,
            estimated_repair_cost: value.estimated_repair_cost,
            odometer: value.odometer,
            odometer_status: value.odometer_status,
            engine_name: value.engine_name,
            engine_cylinders: value.engine_cylinders,
            currency: value.currency,
            sale_date: value.sale_date,
            main_damage: value.main_damage,
            other_damage: value.other_damage,
            country: value.country,
            state: value.state,
            transmission: value.transmission,
            color: value.color,
            fuel_type: value.fuel_type,
            drive_type: value.drive_type,
            keys_status: value.keys_status,
        }
    }
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LotImage {
    pub standard_bucket_key: Option<String>,
    pub standard_mime_type: Option<String>,

    pub thumbnail_bucket_key: Option<String>,
    pub thumbnail_mime_type: Option<String>,

    pub high_res_bucket_key: Option<String>,
    pub high_res_mime_type: Option<String>,

    pub sequence_number: i32,
}

impl From<common::persistence::models::copart::LotImage> for LotImage {
    fn from(value: common::persistence::models::copart::LotImage) -> Self {
        Self {
            standard_bucket_key: value.standard_bucket_key,
            standard_mime_type: value.standard_mime_type,
            thumbnail_bucket_key: value.thumbnail_bucket_key,
            thumbnail_mime_type: value.thumbnail_mime_type,
            high_res_bucket_key: value.high_res_bucket_key,
            high_res_mime_type: value.high_res_mime_type,
            sequence_number: value.sequence_number,
        }
    }
}
