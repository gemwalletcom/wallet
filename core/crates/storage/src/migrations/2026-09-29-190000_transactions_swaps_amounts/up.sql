ALTER TABLE transactions_swaps ADD COLUMN from_amount float NOT NULL DEFAULT 0, ADD COLUMN to_amount float NOT NULL DEFAULT 0;

ALTER TABLE transactions_swaps ALTER COLUMN from_amount DROP DEFAULT, ALTER COLUMN to_amount DROP DEFAULT;
