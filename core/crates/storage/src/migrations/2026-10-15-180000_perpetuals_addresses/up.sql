CREATE TABLE perpetuals_addresses (
    provider perpetual_provider NOT NULL,
    address_id INTEGER NOT NULL REFERENCES wallets_addresses (id) ON DELETE CASCADE,
    referred_at TIMESTAMP NOT NULL,
    cumulative_volume_usd DOUBLE PRECISION NOT NULL,
    cumulative_referral_earned_usd DOUBLE PRECISION NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    PRIMARY KEY (provider, address_id)
);
