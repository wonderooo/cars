pub mod copart {
    use crate::bucket;
    use common::io::copart::LotVehicleVector;
    use diesel::prelude::*;
    use serde::Serialize;
    use std::time::SystemTime;

    #[derive(Queryable, Selectable, Identifiable, Serialize)]
    #[diesel(table_name = crate::orm::schema::lot_vehicle)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    pub struct LotVehicle {
        pub id: i32,
        pub lot_number: i32,
        pub make: String,
        pub year: i32,
    }

    #[derive(Insertable)]
    #[diesel(table_name = crate::orm::schema::lot_vehicle)]
    pub struct NewLotVehicle {
        pub lot_number: i32,
        pub make: String,
        pub year: i32,
    }

    pub struct NewLotVehicles(pub Vec<NewLotVehicle>);

    impl From<LotVehicleVector> for NewLotVehicles {
        fn from(value: LotVehicleVector) -> Self {
            Self(
                value
                    .0
                    .into_iter()
                    .map(|v| NewLotVehicle {
                        lot_number: v.lot_number,
                        make: v.make,
                        year: v.year,
                    })
                    .collect(),
            )
        }
    }

    #[derive(Selectable, Queryable, Associations, Identifiable, Serialize, Debug)]
    #[diesel(table_name = crate::orm::schema::lot_image)]
    #[diesel(belongs_to(LotVehicle, foreign_key = lot_vehicle_number))]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    pub struct LotImage {
        pub id: i32,

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

        pub created_at: SystemTime,
        pub updated_at: SystemTime,

        pub lot_vehicle_number: i32,
    }

    #[derive(Insertable)]
    #[diesel(table_name = crate::orm::schema::lot_image)]
    pub struct NewLotImage {
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

        pub lot_vehicle_number: i32,
    }

    pub struct NewLotImages(pub Vec<NewLotImage>);

    impl From<bucket::models::NewLotImages> for NewLotImages {
        fn from(value: bucket::models::NewLotImages) -> Self {
            Self(
                value
                    .0
                    .into_iter()
                    .map(|bucket| {
                        let (standard_bucket_key, standard_mime_type, standard_source_url) =
                            match bucket.standard {
                                Some(s) => (Some(s.bucket_key), Some(s.mime_type), Some(s.url)),
                                None => (None, None, None),
                            };

                        let (thumbnail_bucket_key, thumbnail_mime_type, thumbnail_source_url) =
                            match bucket.thumbnail {
                                Some(s) => (Some(s.bucket_key), Some(s.mime_type), Some(s.url)),
                                None => (None, None, None),
                            };

                        let (high_res_bucket_key, high_res_mime_type, high_res_source_url) =
                            match bucket.high_res {
                                Some(s) => (Some(s.bucket_key), Some(s.mime_type), Some(s.url)),
                                None => (None, None, None),
                            };

                        NewLotImage {
                            standard_bucket_key,
                            standard_mime_type,
                            standard_source_url,

                            thumbnail_bucket_key,
                            thumbnail_mime_type,
                            thumbnail_source_url,

                            high_res_bucket_key,
                            high_res_mime_type,
                            high_res_source_url,

                            sequence_number: bucket.sequence_number,
                            image_type: bucket.image_type,
                            lot_vehicle_number: bucket.lot_vehicle_number,
                        }
                    })
                    .collect(),
            )
        }
    }
}
