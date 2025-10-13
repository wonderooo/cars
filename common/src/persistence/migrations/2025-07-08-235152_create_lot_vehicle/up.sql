CREATE TABLE lot_vehicle
(
    id                     SERIAL PRIMARY KEY,
    lot_number             INTEGER          NOT NULL,
    make                   VARCHAR          NOT NULL,
    model                  VARCHAR          NOT NULL,
    year                   INTEGER          NOT NULL,
    vehicle_type           VARCHAR          NOT NULL,
    vin                    VARCHAR,
    estimated_retail_value DOUBLE PRECISION NOT NULL,
    estimated_repair_cost  DOUBLE PRECISION NOT NULL,
    odometer               DOUBLE PRECISION NOT NULL,
    odometer_status        VARCHAR,
    engine_name            VARCHAR,
    engine_cylinders       VARCHAR,
    currency               VARCHAR          NOT NULL,
    sale_date              TIMESTAMP,
    main_damage            VARCHAR          NOT NULL,
    other_damage           VARCHAR,
    country                VARCHAR          NOT NULL,
    state                  VARCHAR          NOT NULL,
    transmission           VARCHAR,
    color                  VARCHAR          NOT NULL,
    fuel_type              VARCHAR,
    drive_type             VARCHAR,
    keys_status            VARCHAR,
    created_at             TIMESTAMP        NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at             TIMESTAMP        NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX unique_lot_number ON lot_vehicle (lot_number);

SELECT diesel_manage_updated_at('lot_vehicle');