CREATE TABLE lot_image
(
    id             SERIAL PRIMARY KEY,
    blob_standard  TEXT,
    blob_thumbnail TEXT,
    blob_high_res  TEXT,
    lot_vehicle_id INTEGER NOT NULL REFERENCES lot_vehicle (id)
)
