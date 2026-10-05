CREATE TYPE perpetual_direction AS ENUM ('long', 'short');

CREATE TABLE transactions_perpetuals (
    transaction_id BIGINT PRIMARY KEY REFERENCES transactions (id) ON DELETE CASCADE,
    provider perpetual_provider NOT NULL,
    asset_id VARCHAR NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    kind transaction_type NOT NULL,
    direction perpetual_direction NOT NULL,
    size_usd float NOT NULL,
    pnl_usd float NOT NULL,
    referral_fee_amount_usd float NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('transactions_perpetuals');

CREATE INDEX transactions_perpetuals_provider_idx ON transactions_perpetuals (provider);
CREATE INDEX transactions_perpetuals_created_at_idx ON transactions_perpetuals (created_at DESC);
