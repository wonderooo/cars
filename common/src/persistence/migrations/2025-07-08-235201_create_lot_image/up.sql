CREATE TABLE lot_image
(
    id                   SERIAL PRIMARY KEY,

    standard_bucket_key  VARCHAR,
    standard_mime_type   VARCHAR,
    standard_source_url  VARCHAR,

    thumbnail_bucket_key VARCHAR,
    thumbnail_mime_type  VARCHAR,
    thumbnail_source_url VARCHAR,

    high_res_bucket_key  VARCHAR,
    high_res_mime_type   VARCHAR,
    high_res_source_url  VARCHAR,

    sequence_number      INTEGER   NOT NULL,
    image_type           VARCHAR   NOT NULL,

    created_at           TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at           TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    lot_vehicle_number   INTEGER   NOT NULL REFERENCES lot_vehicle (lot_number)
);

SELECT diesel_manage_updated_at('lot_image');