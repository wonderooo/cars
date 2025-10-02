CREATE TABLE lot_vehicle
(
    id         SERIAL PRIMARY KEY,
    lot_number INTEGER NOT NULL,
    make       VARCHAR NOT NULL,
    year       INTEGER NOT NULL
);

CREATE UNIQUE INDEX unique_lot_number ON lot_vehicle (lot_number);