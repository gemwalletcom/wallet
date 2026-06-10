CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TYPE api_client_scope AS ENUM (
    'devices_read',
    'devices_subscriptions_read',
    'devices_transactions_read',
    'webhooks_transactions',
    'webhooks_support',
    'webhooks_fiat'
);

CREATE TABLE api_clients (
    id SERIAL PRIMARY KEY,
    name VARCHAR(128) NOT NULL UNIQUE,
    secret VARCHAR(64) NOT NULL DEFAULT gen_random_uuid()::text,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

CREATE TABLE api_client_scopes (
    client_id INTEGER NOT NULL REFERENCES api_clients(id) ON DELETE CASCADE,
    scope api_client_scope NOT NULL,
    resource VARCHAR(128) NOT NULL DEFAULT '',
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    PRIMARY KEY (client_id, scope, resource)
);

CREATE UNIQUE INDEX api_clients_secret_idx ON api_clients (secret);

SELECT diesel_manage_updated_at('api_clients');
SELECT diesel_manage_updated_at('api_client_scopes');
