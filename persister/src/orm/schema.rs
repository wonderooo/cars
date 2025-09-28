// @generated automatically by Diesel CLI.

diesel::table! {
    lot_image (id) {
        id -> Int4,
        standard_bucket_key -> Nullable<Varchar>,
        standard_mime_type -> Nullable<Varchar>,
        standard_source_url -> Nullable<Varchar>,
        thumbnail_bucket_key -> Nullable<Varchar>,
        thumbnail_mime_type -> Nullable<Varchar>,
        thumbnail_source_url -> Nullable<Varchar>,
        high_res_bucket_key -> Nullable<Varchar>,
        high_res_mime_type -> Nullable<Varchar>,
        high_res_source_url -> Nullable<Varchar>,
        sequence_number -> Int4,
        image_type -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        lot_vehicle_number -> Int4,
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

diesel::allow_tables_to_appear_in_same_query!(
    lot_image,
    lot_vehicle,
);
