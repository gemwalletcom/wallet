CREATE TYPE asset_association_type AS ENUM ('official', 'bridged', 'wrapped');

CREATE TABLE assets_associations (
    asset_id VARCHAR(128) PRIMARY KEY REFERENCES assets (id) ON DELETE CASCADE,
    id VARCHAR(64) NOT NULL,
    association_type asset_association_type NOT NULL
);

CREATE INDEX assets_associations_id_idx ON assets_associations (id);
