CREATE TABLE IF NOT EXISTS charts (
    coin_id VARCHAR(255) NOT NULL REFERENCES prices (id) ON DELETE CASCADE,
    price float NOT NULL,
    created_at TIMESTAMP NOT NULL,
    PRIMARY KEY (coin_id, created_at)
);

CREATE TABLE IF NOT EXISTS charts_hourly (
    coin_id VARCHAR(255) NOT NULL REFERENCES prices (id) ON DELETE CASCADE,
    price float NOT NULL,
    created_at TIMESTAMP NOT NULL,
    PRIMARY KEY (coin_id, created_at)
);

CREATE TABLE IF NOT EXISTS charts_daily (
    coin_id VARCHAR(255) NOT NULL REFERENCES prices (id) ON DELETE CASCADE,
    price float NOT NULL,
    created_at TIMESTAMP NOT NULL,
    PRIMARY KEY (coin_id, created_at)
);

-- indexes
CREATE INDEX IF NOT EXISTS idx_charts_created_at ON charts (created_at);
CREATE INDEX IF NOT EXISTS idx_charts_hourly_created_at ON charts_hourly (created_at);
CREATE INDEX IF NOT EXISTS idx_charts_daily_created_at ON charts_daily (created_at);

-- functions
DROP FUNCTION IF EXISTS aggregate_hourly_charts();
DROP FUNCTION IF EXISTS aggregate_hourly_charts(VARCHAR[]);
CREATE OR REPLACE FUNCTION aggregate_hourly_charts(_price_ids VARCHAR[] DEFAULT NULL, _created_at TIMESTAMP DEFAULT NULL) RETURNS VOID AS $$
BEGIN
    INSERT INTO charts_hourly (coin_id, created_at, price)
    SELECT
        charts.coin_id,
        DATE_TRUNC('hour', charts.created_at),
        AVG(charts.price)
    FROM charts
    WHERE charts.created_at >= COALESCE(DATE_TRUNC('hour', _created_at), DATE_TRUNC('hour', NOW()) - INTERVAL '1 hour')
      AND charts.created_at < COALESCE(DATE_TRUNC('hour', _created_at) + INTERVAL '1 hour', DATE_TRUNC('hour', NOW()) + INTERVAL '1 hour')
      AND (_price_ids IS NULL OR charts.coin_id = ANY(_price_ids))
    GROUP BY charts.coin_id, DATE_TRUNC('hour', charts.created_at)
    ON CONFLICT (coin_id, created_at) DO UPDATE SET price = EXCLUDED.price
    WHERE charts_hourly.price <> EXCLUDED.price;
END;
$$ LANGUAGE plpgsql;

DROP FUNCTION IF EXISTS aggregate_daily_charts();
DROP FUNCTION IF EXISTS aggregate_daily_charts(VARCHAR[]);
CREATE OR REPLACE FUNCTION aggregate_daily_charts(_price_ids VARCHAR[] DEFAULT NULL, _created_at TIMESTAMP DEFAULT NULL) RETURNS VOID AS $$
BEGIN
    INSERT INTO charts_daily (coin_id, created_at, price)
    SELECT
        charts_hourly.coin_id,
        DATE_TRUNC('day', charts_hourly.created_at),
        AVG(charts_hourly.price)
    FROM charts_hourly
    WHERE charts_hourly.created_at >= COALESCE(DATE_TRUNC('day', _created_at), DATE_TRUNC('day', NOW()) - INTERVAL '1 day')
      AND charts_hourly.created_at < COALESCE(DATE_TRUNC('day', _created_at) + INTERVAL '1 day', DATE_TRUNC('day', NOW()) + INTERVAL '1 day')
      AND (_price_ids IS NULL OR charts_hourly.coin_id = ANY(_price_ids))
    GROUP BY charts_hourly.coin_id, DATE_TRUNC('day', charts_hourly.created_at)
    ON CONFLICT (coin_id, created_at) DO UPDATE SET price = EXCLUDED.price
    WHERE charts_daily.price <> EXCLUDED.price;
END;
$$ LANGUAGE plpgsql;

ALTER TABLE charts SET (autovacuum_vacuum_scale_factor = 0.02, autovacuum_vacuum_threshold = 1000);
ALTER TABLE charts_hourly SET (autovacuum_vacuum_scale_factor = 0.02, autovacuum_vacuum_threshold = 500);
ALTER TABLE charts_daily SET (autovacuum_vacuum_scale_factor = 0.02, autovacuum_vacuum_threshold = 500);
