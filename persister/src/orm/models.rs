pub mod copart {
    use diesel::prelude::*;
    use std::ops::{Deref, DerefMut};

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

    impl Deref for NewLotVehicles {
        type Target = Vec<NewLotVehicle>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for NewLotVehicles {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl Into<NewLotVehicles> for browser::response::lot_search::ApiResponse {
        fn into(self) -> NewLotVehicles {
            NewLotVehicles(
                self.data
                    .results
                    .content
                    .into_iter()
                    .map(|l| NewLotVehicle {
                        lot_number: l.ln as i32,
                        make: l.mkn,
                        year: l.lcy,
                    })
                    .collect(),
            )
        }
    }

    pub type Base64Blob = String;

    #[derive(Selectable, Queryable, Associations)]
    #[diesel(table_name = crate::orm::schema::lot_image)]
    #[diesel(belongs_to(LotVehicle))]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    pub struct LotImage {
        pub id: i32,
        pub blob_standard: Option<Base64Blob>,
        pub blob_thumbnail: Option<Base64Blob>,
        pub blob_high_res: Option<Base64Blob>,
        pub lot_vehicle_id: i32,
    }

    #[derive(Insertable)]
    #[diesel(table_name = crate::orm::schema::lot_image)]
    pub struct NewLotImage {
        pub blob_standard: Option<Base64Blob>,
        pub blob_thumbnail: Option<Base64Blob>,
        pub blob_high_res: Option<Base64Blob>,
        pub lot_vehicle_id: i32,
    }
}
