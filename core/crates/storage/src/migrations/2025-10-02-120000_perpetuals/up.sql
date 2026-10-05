CREATE TYPE perpetual_provider AS ENUM ('hypercore');

CREATE TABLE perpetuals (
    id VARCHAR(128) PRIMARY KEY,
    name VARCHAR(128) NOT NULL,
    provider perpetual_provider NOT NULL,
    asset_id VARCHAR(256) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    associated_asset_id VARCHAR(256) REFERENCES assets (id) ON DELETE SET NULL,
    identifier VARCHAR(128) NOT NULL,
    price DOUBLE PRECISION NOT NULL,
    price_percent_change_24h DOUBLE PRECISION NOT NULL,
    open_interest DOUBLE PRECISION NOT NULL,
    volume_24h DOUBLE PRECISION NOT NULL,
    funding DOUBLE PRECISION NOT NULL,
    leverage INTEGER[] NOT NULL,
    is_isolated_only BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT diesel_manage_updated_at('perpetuals');
CREATE INDEX perpetuals_updated_at_idx ON perpetuals (updated_at);
