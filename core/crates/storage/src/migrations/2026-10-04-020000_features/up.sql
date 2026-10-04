CREATE TYPE feature AS ENUM ('buy', 'sell', 'swap', 'perpetuals', 'rewards');

CREATE TABLE features (
    id feature NOT NULL,
    alpha2 VARCHAR(2) NOT NULL REFERENCES countries(alpha2),
    is_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (id, alpha2)
);
