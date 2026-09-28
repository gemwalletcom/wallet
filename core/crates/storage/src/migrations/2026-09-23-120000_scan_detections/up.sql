CREATE TYPE scan_type AS ENUM ('address', 'address_poisoning', 'website', 'asset');
CREATE TYPE scan_provider AS ENUM ('goplus', 'hashdit', 'tronscan');

CREATE TABLE scan_detections (
    id SERIAL PRIMARY KEY,
    scan_type scan_type NOT NULL,
    chain VARCHAR REFERENCES chains (id) ON DELETE CASCADE,
    target VARCHAR(256) NOT NULL,
    provider scan_provider NOT NULL,
    reason VARCHAR,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    UNIQUE NULLS NOT DISTINCT (scan_type, chain, target)
);

SELECT diesel_manage_updated_at('scan_detections');

CREATE INDEX scan_detections_target_idx ON scan_detections (target);
