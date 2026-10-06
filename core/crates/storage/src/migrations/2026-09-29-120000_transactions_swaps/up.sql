CREATE TYPE swap_provider AS ENUM ('uniswap_v3', 'uniswap_v4', 'pancakeswap_v3', 'aerodrome', 'panora', 'thorchain', 'jupiter', 'okx', 'across', 'oku', 'wagmi', 'stonfi_v2', 'mayan', 'chainflip', 'near_intents', 'cetus_clmm', 'relay', 'hyperliquid', 'orca', 'squid', 'mayachain', 'swaps_xyz');

CREATE TYPE swap_status AS ENUM ('pending', 'completed', 'failed', 'refunded');

CREATE TABLE transactions_swaps (
    transaction_id BIGINT PRIMARY KEY REFERENCES transactions (id) ON DELETE CASCADE,
    provider swap_provider NOT NULL,
    status swap_status NOT NULL,
    from_asset_id VARCHAR(128) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    from_amount float NOT NULL,
    from_amount_usd float,
    to_asset_id VARCHAR(128) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    to_amount float NOT NULL,
    to_amount_usd float,
    referral_fee_asset_id VARCHAR(128) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    referral_fee_amount_usd float,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('transactions_swaps');

CREATE INDEX transactions_swaps_provider_idx ON transactions_swaps (provider);
CREATE INDEX transactions_swaps_created_at_idx ON transactions_swaps (created_at DESC);
