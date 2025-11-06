ALTER TABLE lot_vehicle DROP CONSTRAINT lot_vehicle_pkey;
ALTER TABLE lot_vehicle DROP COLUMN id;
ALTER TABLE lot_vehicle ADD PRIMARY KEY (lot_number);