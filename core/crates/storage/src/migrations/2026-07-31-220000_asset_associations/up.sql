CREATE TYPE asset_association_type AS ENUM ('official', 'bridged', 'wrapped');

CREATE TABLE assets_associations (
    asset_id VARCHAR(128) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    associated_asset_id VARCHAR(128) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    association_type asset_association_type NOT NULL,
    PRIMARY KEY (asset_id, associated_asset_id),
    CHECK (asset_id <> associated_asset_id)
);
