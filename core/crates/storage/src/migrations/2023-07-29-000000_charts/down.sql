-- Drop functions
DROP FUNCTION IF EXISTS aggregate_hourly_charts(VARCHAR[]);
DROP FUNCTION IF EXISTS aggregate_daily_charts(VARCHAR[]);

-- Drop tables
DROP TABLE IF EXISTS charts_daily;
DROP TABLE IF EXISTS charts_hourly;
DROP TABLE IF EXISTS charts;
