CREATE TABLE scan_websites (
    host VARCHAR(253) PRIMARY KEY,
    is_fraudulent boolean NOT NULL DEFAULT true,
    updated_at timestamp NOT NULL DEFAULT current_timestamp,
    created_at timestamp NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('scan_websites');
