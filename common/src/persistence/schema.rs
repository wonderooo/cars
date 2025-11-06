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
    lot_vehicle (lot_number) {
        lot_number -> Int4,
        make -> Varchar,
        model -> Varchar,
        year -> Int4,
        vehicle_type -> Varchar,
        vin -> Nullable<Varchar>,
        estimated_retail_value -> Float8,
        estimated_repair_cost -> Float8,
        odometer -> Float8,
        odometer_status -> Nullable<Varchar>,
        engine_name -> Nullable<Varchar>,
        engine_cylinders -> Nullable<Varchar>,
        currency -> Varchar,
        sale_date -> Nullable<Timestamp>,
        main_damage -> Varchar,
        other_damage -> Nullable<Varchar>,
        country -> Varchar,
        state -> Varchar,
        transmission -> Nullable<Varchar>,
        color -> Varchar,
        fuel_type -> Nullable<Varchar>,
        drive_type -> Nullable<Varchar>,
        keys_status -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(lot_image -> lot_vehicle (lot_vehicle_number));

diesel::allow_tables_to_appear_in_same_query!(
    lot_image,
    lot_vehicle,
);
