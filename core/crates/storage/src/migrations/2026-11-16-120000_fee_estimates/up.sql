CREATE TABLE IF NOT EXISTS fee_estimates (
    chain VARCHAR(32) NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
    priority VARCHAR(16) NOT NULL,
    gas_price FLOAT8 NOT NULL,
    priority_fee FLOAT8 NOT NULL,
    created_at TIMESTAMP NOT NULL,
    PRIMARY KEY (chain, priority, created_at)
);
