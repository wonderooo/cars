pub mod error {
    use diesel::result::Error;
    use diesel_async::pooled_connection::deadpool::PoolError;
    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    #[derive(Debug, Error, Serialize, Deserialize)]
    pub enum GeneralError {
        #[error("chromium oxide error: {0}")]
        CdpError(String),
        #[error("could not build cdp command from given arguments")]
        CdpCommandBuild(String),
        #[error("argument is not valid utf8: {0}")]
        InvalidUtf8(String),
        #[error("could not marshall/unmarshall given argument: {0}")]
        Json(String),
        #[error("could not send copart browser cmd/response to channel")]
        ChannelSend,
        #[error("intercepted browser request/response which is not handled")]
        UnhandledInterception(String),
        #[error("correlation id not found in url")]
        CorrelationIdNotFound(String),
        #[error("page number not found in query params")]
        PageNumberNotFound,
        #[error("lot number not found in query params")]
        LotNumberNotFound,
        #[error("could not decode to base64 from given argument: {0}")]
        Base64Decode(String),
        #[error("could not build valid URL from given argument")]
        InvalidUrl(String),
        #[error("could not parse to int: {0}")]
        ParseInt(String),
        #[error("browser worker pool is empty")]
        BrowserPoolEmpty,
        #[error("postgres pool error: `{0}`")]
        PgPool(String),
        #[error("diesel error: `{0}`")]
        Diesel(String),
        #[error("smf error: `{0}`")]
        Smf(String),
        #[error("s3 error: `{0}`")]
        S3(String),
    }

    impl From<std::num::ParseIntError> for GeneralError {
        fn from(value: std::num::ParseIntError) -> Self {
            Self::ParseInt(value.to_string())
        }
    }

    impl From<url::ParseError> for GeneralError {
        fn from(value: url::ParseError) -> Self {
            Self::InvalidUrl(value.to_string())
        }
    }

    impl From<chromiumoxide::error::CdpError> for GeneralError {
        fn from(value: chromiumoxide::error::CdpError) -> Self {
            Self::CdpError(value.to_string())
        }
    }

    impl From<base64::DecodeError> for GeneralError {
        fn from(value: base64::DecodeError) -> Self {
            Self::Base64Decode(value.to_string())
        }
    }

    impl From<serde_json::Error> for GeneralError {
        fn from(value: serde_json::Error) -> Self {
            Self::Json(value.to_string())
        }
    }

    impl From<std::str::Utf8Error> for GeneralError {
        fn from(value: std::str::Utf8Error) -> Self {
            Self::InvalidUtf8(value.to_string())
        }
    }

    impl From<PoolError> for GeneralError {
        fn from(value: PoolError) -> Self {
            Self::PgPool(value.to_string())
        }
    }

    impl From<diesel::result::Error> for GeneralError {
        fn from(value: Error) -> Self {
            Self::Diesel(value.to_string())
        }
    }

    impl<T> From<tokio::sync::mpsc::error::SendError<T>> for GeneralError {
        fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
            Self::ChannelSend
        }
    }
}

pub mod copart {
    use crate::count_some_none;
    use crate::io::error::GeneralError;
    use crate::kafka::ToTopic;
    use serde::{Deserialize, Serialize};
    use std::fmt::{Debug, Formatter};

    pub type LotNumber = i32;
    pub type PageNumber = usize;
    pub type AuctionId = String;
    pub type Base64Blob = String;
    pub type DateTimeRfc3339 = String;
    pub type LotYear = usize;

    #[derive(Debug, Serialize, Deserialize)]
    pub enum CopartCmd {
        /// Sent by `sched` periodically, received by `browser` to fetch raw data from the provider
        LotSearch {
            page_number: PageNumber,
            date_start: DateTimeRfc3339,
            date_end: DateTimeRfc3339,
            year_start: LotYear,
            year_end: LotYear,
        },
        /// Sent by `persister` after lot search response has been received, received by `browser`
        /// to fetch image urls from the provider
        LotImages(LotNumber),
        Auction(AuctionId),
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum CopartResponse {
        /// Sent by `browser` after lot search cmd has been received, it includes raw data
        /// from the provider of lot vehicles for a specified page number, received by `persister`
        LotSearch(Result<LotSearchResponse, GeneralError>),
        /// Sent by `browser` after lot images cmd has been received, it includes raw data
        /// from the provider of single lot vehicle for specified lot number, received by `imgsync`
        LotImages(Result<LotImagesResponse, GeneralError>),
        SyncedImages(Result<SyncedImagesResponse, GeneralError>),
    }

    impl ToTopic for CopartCmd {
        fn to_topic(&self) -> String {
            match self {
                Self::LotSearch { .. } => "copart_cmd_lot_search".to_string(),
                Self::LotImages(..) => "copart_cmd_lot_images".to_string(),
                Self::Auction(_) => "copart_cmd_auction".to_string(),
            }
        }
    }

    impl ToTopic for CopartResponse {
        fn to_topic(&self) -> String {
            match self {
                Self::LotSearch { .. } => "copart_response_lot_search".to_string(),
                Self::LotImages { .. } => "copart_response_lot_images".to_string(),
                Self::SyncedImages { .. } => "copart_response_synced_images".to_string(),
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct LotSearchResponse {
        pub page_number: PageNumber,
        pub response: LotVehicleVector,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct LotImagesResponse {
        pub lot_number: LotNumber,
        pub response: LotImagesVector,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SyncedImagesResponse {
        pub lot_number: LotNumber,
        pub response: SyncedImagesVector,
    }

    #[derive(Serialize, Deserialize)]
    pub struct LotVehicleVector(pub Vec<LotVehicle>);

    #[derive(Debug, Serialize, Deserialize)]
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

    #[derive(Serialize, Deserialize)]
    pub struct LotImagesVector(pub Vec<LotImages>);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct LotImages {
        pub thumbnail_url: Option<String>,
        pub full_url: Option<String>,
        pub high_res_url: Option<String>,
        pub sequence_number: i32,
        pub image_type: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SyncedImagesVector(pub Vec<SyncedImages>);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SyncedImages {
        pub standard_bucket_key: Option<String>,
        pub standard_mime_type: Option<String>,
        pub standard_source_url: Option<String>,

        pub thumbnail_bucket_key: Option<String>,
        pub thumbnail_mime_type: Option<String>,
        pub thumbnail_source_url: Option<String>,

        pub high_res_bucket_key: Option<String>,
        pub high_res_mime_type: Option<String>,
        pub high_res_source_url: Option<String>,

        pub sequence_number: i32,
        pub image_type: String,
    }

    impl Debug for LotVehicleVector {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "n lot vehicles: `{}`", self.0.len())
        }
    }

    impl Debug for LotImagesVector {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let (some_thumbnail, none_thumbnail) =
                count_some_none(&self.0, |i| i.thumbnail_url.as_deref());
            let (some_high, none_high) = count_some_none(&self.0, |i| i.high_res_url.as_deref());
            let (some_full, none_full) = count_some_none(&self.0, |i| i.full_url.as_deref());

            write!(
                f,
                "thumbnail_url {{some: {some_thumbnail}, none: {none_thumbnail}}}, high_res_url {{some: {some_high}, none: {none_high}}}, full_url {{some: {some_full}, none: {none_full}}}",
            )
        }
    }
}
