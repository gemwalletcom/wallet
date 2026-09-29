CREATE TYPE swap_provider AS ENUM (
    'uniswap_v3', 'uniswap_v4', 'pancakeswap_v3', 'aerodrome', 'panora', 'thorchain', 'jupiter', 'okx', 'across', 'oku', 'wagmi',
    'stonfi_v2', 'mayan', 'chainflip', 'near_intents', 'cetus_clmm', 'relay', 'hyperliquid', 'orca', 'squid', 'mayachain', 'swaps_xyz'
);
CREATE TYPE swap_status AS ENUM ('pending', 'completed', 'failed', 'refunded');

CREATE TABLE swap_partner_transactions (
    id SERIAL PRIMARY KEY,
    provider swap_provider NOT NULL,
    provider_transaction_id VARCHAR(256) NOT NULL,
    status swap_status NOT NULL,
    from_asset_id VARCHAR(128) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    from_value VARCHAR(256) NOT NULL,
    from_amount_usd float,
    to_asset_id VARCHAR(128) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    to_value VARCHAR(256) NOT NULL,
    to_amount_usd float,
    referral_fee_asset_id VARCHAR(128) REFERENCES assets (id) ON DELETE CASCADE,
    referral_fee_value VARCHAR(256),
    referral_fee_amount_usd float,
    from_transaction_hash VARCHAR(256),
    to_transaction_hash VARCHAR(256),
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    UNIQUE (provider, provider_transaction_id)
);

SELECT diesel_manage_updated_at('swap_partner_transactions');
