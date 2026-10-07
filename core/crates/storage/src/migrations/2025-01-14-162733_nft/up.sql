CREATE TYPE nft_type AS ENUM ('erc721', 'erc1155', 'spl', 'jetton');

CREATE TABLE nft_collections (
    id SERIAL PRIMARY KEY,
    identifier VARCHAR(512) UNIQUE NOT NULL,

    chain VARCHAR(32) NOT NULL REFERENCES chains (id) ON DELETE CASCADE,

    name VARCHAR(1024) NOT NULL,
    description VARCHAR(4096) NOT NULL,
    symbol VARCHAR(128),

    contract_address VARCHAR(128) NOT NULL,

    image_preview_url VARCHAR(512) NOT NULL,
    image_preview_mime_type VARCHAR(64) NOT NULL,

    is_verified BOOLEAN NOT NULL default false,
    is_enabled BOOLEAN NOT NULL default true,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('nft_collections');
CREATE INDEX nft_collections_updated_at_idx ON nft_collections (updated_at);

CREATE TABLE nft_collections_links (
    id SERIAL PRIMARY KEY,

    collection_id INTEGER NOT NULL REFERENCES nft_collections (id) ON DELETE CASCADE,

    link_type link_type NOT NULL,

    url VARCHAR(256) NOT NULL,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,

    UNIQUE(collection_id, link_type)
);

SELECT diesel_manage_updated_at('nft_collections_links');

CREATE TABLE nft_assets (
    id SERIAL PRIMARY KEY,
    identifier VARCHAR(512) UNIQUE NOT NULL,

    collection_id INTEGER NOT NULL REFERENCES nft_collections (id) ON DELETE CASCADE,
    chain VARCHAR(32) NOT NULL REFERENCES chains (id) ON DELETE CASCADE,

    name VARCHAR(1024) NOT NULL,
    description VARCHAR(4096) NOT NULL,

    image_preview_url VARCHAR(512) NOT NULL,
    image_preview_mime_type VARCHAR(64) NOT NULL,

    resource_url VARCHAR(512) NOT NULL,
    resource_mime_type VARCHAR(64) NOT NULL,

    token_type nft_type NOT NULL,
    token_id VARCHAR(512) NOT NULL,
    contract_address VARCHAR(512) NOT NULL,

    attributes JSONB NOT NULL,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('nft_assets');
CREATE INDEX nft_assets_collection_id_idx ON nft_assets (collection_id);

CREATE TABLE nft_assets_associations (
    id SERIAL PRIMARY KEY,

    address_id INTEGER NOT NULL REFERENCES wallets_addresses (id) ON DELETE CASCADE,
    nft_asset_id INTEGER NOT NULL REFERENCES nft_assets (id) ON DELETE CASCADE,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,

    UNIQUE(address_id, nft_asset_id)
);

SELECT diesel_manage_updated_at('nft_assets_associations');
CREATE INDEX nft_assets_associations_nft_asset_id_idx ON nft_assets_associations (nft_asset_id);

CREATE TABLE nft_reports (
    id SERIAL PRIMARY KEY,

    nft_asset_id INTEGER REFERENCES nft_assets (id) ON DELETE CASCADE,
    collection_id INTEGER NOT NULL REFERENCES nft_collections (id) ON DELETE CASCADE,

    device_id INTEGER NOT NULL REFERENCES devices (id) ON DELETE CASCADE,
    reason VARCHAR(1024),
    reviewed BOOLEAN NOT NULL DEFAULT false,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('nft_reports');
