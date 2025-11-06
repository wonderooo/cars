pub mod copart {
    use crate::bucket;
    use crate::io::copart::LotVehicleVector;
    use diesel::prelude::*;

    #[derive(Queryable, Selectable, Identifiable)]
    #[diesel(table_name = crate::persistence::schema::lot_vehicle)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    #[diesel(primary_key(lot_number))]
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
        pub created_at: chrono::NaiveDateTime,
        pub updated_at: chrono::NaiveDateTime,
    }

    #[derive(Insertable)]
    #[diesel(table_name = crate::persistence::schema::lot_vehicle)]
    pub struct NewLotVehicle {
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
                        model: v.model,
                        year: v.year,
                        vehicle_type: v.vehicle_type,
                        vin: v.vin,
                        estimated_retail_value: v.estimated_retail_value,
                        estimated_repair_cost: v.estimated_repair_cost,
                        odometer: v.odometer,
                        odometer_status: v.odometer_status,
                        engine_name: v.engine_name,
                        engine_cylinders: v.engine_cylinders,
                        currency: v.currency,
                        sale_date: v.sale_date,
                        main_damage: v.main_damage,
                        other_damage: v.other_damage,
                        country: v.country,
                        state: v.state,
                        transmission: v.transmission,
                        color: v.color,
                        fuel_type: v.fuel_type,
                        drive_type: v.drive_type,
                        keys_status: v.keys_status,
                    })
                    .collect(),
            )
        }
    }

    #[derive(Selectable, Queryable, Associations, Identifiable)]
    #[diesel(table_name = crate::persistence::schema::lot_image)]
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

        pub created_at: chrono::NaiveDateTime,
        pub updated_at: chrono::NaiveDateTime,

        pub lot_vehicle_number: i32,
    }

    #[derive(Insertable)]
    #[diesel(table_name = crate::persistence::schema::lot_image)]
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
