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
    pub year: i32,
}

impl From<common::persistence::models::copart::LotVehicle> for LotVehicle {
    fn from(value: common::persistence::models::copart::LotVehicle) -> Self {
        Self {
            lot_number: value.lot_number,
            make: value.make,
            year: value.year,
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
