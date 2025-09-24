pub mod copart {
    use common::io;
    use common::io::copart::{LotImageBlobsResponse, LotVehicleVector};
    use diesel::prelude::*;

    #[derive(Queryable, Selectable)]
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

    #[derive(Selectable, Queryable, Associations)]
    #[diesel(table_name = crate::orm::schema::lot_image)]
    #[diesel(belongs_to(LotVehicle, foreign_key = lot_vehicle_number))]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    pub struct LotImage {
        pub id: i32,
        pub blob_standard: Option<io::copart::Base64Blob>,
        pub blob_thumbnail: Option<io::copart::Base64Blob>,
        pub blob_high_res: Option<io::copart::Base64Blob>,
        pub lot_vehicle_number: i32,
    }

    #[derive(Insertable)]
    #[diesel(table_name = crate::orm::schema::lot_image)]
    pub struct NewLotImage {
        pub blob_standard: Option<io::copart::Base64Blob>,
        pub blob_thumbnail: Option<io::copart::Base64Blob>,
        pub blob_high_res: Option<io::copart::Base64Blob>,
        pub lot_vehicle_number: i32,
    }

    pub struct NewLotImages(pub Vec<NewLotImage>);

    impl From<LotImageBlobsResponse> for NewLotImages {
        fn from(value: LotImageBlobsResponse) -> Self {
            Self(
                value
                    .response
                    .0
                    .into_iter()
                    .map(|i| NewLotImage {
                        blob_standard: i.standard,
                        blob_thumbnail: i.thumbnail,
                        blob_high_res: i.high_res,
                        lot_vehicle_number: value.lot_number,
                    })
                    .collect(),
            )
        }
    }
}
