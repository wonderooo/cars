// @generated automatically by Diesel CLI.

diesel::table! {
    lot_image (id) {
        id -> Int4,
        blob_standard -> Nullable<Text>,
        blob_thumbnail -> Nullable<Text>,
        blob_high_res -> Nullable<Text>,
        lot_vehicle_id -> Int4,
    }
}

diesel::table! {
    lot_vehicle (id) {
        id -> Int4,
        lot_number -> Int4,
        make -> Varchar,
        year -> Int4,
    }
}

diesel::joinable!(lot_image -> lot_vehicle (lot_vehicle_id));

diesel::allow_tables_to_appear_in_same_query!(
    lot_image,
    lot_vehicle,
);
