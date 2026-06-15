CREATE TABLE perpetuals_tags (
    perpetual_id VARCHAR(128) NOT NULL REFERENCES perpetuals (id) ON DELETE CASCADE,
    tag_id VARCHAR(64) NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
    "order" INTEGER,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (perpetual_id, tag_id)
);
