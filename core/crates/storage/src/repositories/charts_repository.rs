use crate::models::chart::{ChartRow, DailyChartRow, HourlyChartRow};
use crate::models::min_max::{DataPoint, MinMax};
use crate::schema::charts::dsl::{charts, coin_id as raw_coin_id, created_at as raw_created_at, price as raw_price};
use crate::schema::charts_daily::dsl::{charts_daily, coin_id as daily_coin_id, created_at as daily_created_at, price as daily_price};
use crate::schema::charts_hourly::dsl::{charts_hourly, coin_id as hourly_coin_id, created_at as hourly_created_at, price as hourly_price};
use crate::{DatabaseClient, DatabaseError};
use chrono::NaiveDateTime;
use diesel::dsl::sql;
use diesel::prelude::*;
use primitives::{ChartPeriod, ChartTimeframe};

enum ChartGranularity {
    Minute,
    Minute15,
    Hourly,
    Hour6,
    Daily,
}

pub type ChartResult = (chrono::NaiveDateTime, f64);

#[derive(Debug, Clone, PartialEq)]
pub struct ChartPoint {
    pub price_id: String,
    pub price: f64,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub enum ChartFilter {
    CreatedBefore(NaiveDateTime),
    CreatedAfter(NaiveDateTime),
    PriceIds(Vec<String>),
}

pub(crate) fn insert_chart_rows(client: &mut DatabaseClient, timeframe: ChartTimeframe, values: Vec<ChartRow>) -> Result<usize, diesel::result::Error> {
    if values.is_empty() {
        return Ok(0);
    }
    match timeframe {
        ChartTimeframe::Raw => diesel::insert_into(charts).values(values).on_conflict_do_nothing().execute(&mut client.connection),
        ChartTimeframe::Hourly => {
            let rows: Vec<HourlyChartRow> = values.into_iter().map(Into::into).collect();
            diesel::insert_into(charts_hourly).values(rows).on_conflict_do_nothing().execute(&mut client.connection)
        }
        ChartTimeframe::Daily => {
            let rows: Vec<DailyChartRow> = values.into_iter().map(Into::into).collect();
            diesel::insert_into(charts_daily).values(rows).on_conflict_do_nothing().execute(&mut client.connection)
        }
    }
}

pub trait ChartsRepository {
    fn add_charts(&mut self, timeframe: ChartTimeframe, values: Vec<ChartPoint>) -> Result<usize, DatabaseError>;
    fn get_charts(&mut self, price_id: &str, period: &ChartPeriod) -> Result<Vec<ChartResult>, DatabaseError>;
    fn aggregate_charts(&mut self, timeframe: ChartTimeframe) -> Result<usize, DatabaseError>;
    fn delete_charts(&mut self, timeframe: ChartTimeframe, before: NaiveDateTime) -> Result<usize, DatabaseError>;
    fn get_charts_by_filter(&mut self, filters: Vec<ChartFilter>) -> Result<Vec<(String, f64)>, DatabaseError>;
}

impl ChartsRepository for DatabaseClient {
    fn add_charts(&mut self, timeframe: ChartTimeframe, values: Vec<ChartPoint>) -> Result<usize, DatabaseError> {
        let rows = values.into_iter().map(|point| ChartRow::new(point.price_id, point.price, point.created_at)).collect();
        Ok(insert_chart_rows(self, timeframe, rows)?)
    }

    fn get_charts(&mut self, price_id: &str, period: &ChartPeriod) -> Result<Vec<ChartResult>, DatabaseError> {
        let date_selection = format!("date_bin('{}', created_at, timestamp '2000-01-01')", period_sql(*period));
        let granularity = chart_granularity_for_period(period);
        let created_at_filter = format!("created_at >= now() - INTERVAL '{} minutes'", period_minutes(*period));
        Ok(match granularity {
            ChartGranularity::Minute | ChartGranularity::Minute15 => charts
                .select((sql::<diesel::sql_types::Timestamp>(date_selection.as_str()), sql::<diesel::sql_types::Double>("AVG(price)")))
                .filter(raw_coin_id.eq(price_id))
                .filter(sql::<diesel::sql_types::Bool>(&created_at_filter))
                .group_by(sql::<diesel::sql_types::Numeric>("1"))
                .order(sql::<diesel::sql_types::Numeric>("1").asc())
                .load(&mut self.connection)?,
            ChartGranularity::Hourly | ChartGranularity::Hour6 => charts_hourly
                .select((sql::<diesel::sql_types::Timestamp>(date_selection.as_str()), sql::<diesel::sql_types::Double>("AVG(price)")))
                .filter(hourly_coin_id.eq(price_id))
                .filter(sql::<diesel::sql_types::Bool>(&created_at_filter))
                .group_by(sql::<diesel::sql_types::Numeric>("1"))
                .order(sql::<diesel::sql_types::Numeric>("1").asc())
                .load(&mut self.connection)?,
            ChartGranularity::Daily => charts_daily
                .select((sql::<diesel::sql_types::Timestamp>(date_selection.as_str()), sql::<diesel::sql_types::Double>("AVG(price)")))
                .filter(daily_coin_id.eq(price_id))
                .filter(sql::<diesel::sql_types::Bool>(&created_at_filter))
                .group_by(sql::<diesel::sql_types::Numeric>("1"))
                .order(sql::<diesel::sql_types::Numeric>("1").asc())
                .load(&mut self.connection)?,
        })
    }

    fn aggregate_charts(&mut self, timeframe: ChartTimeframe) -> Result<usize, DatabaseError> {
        let query = match timeframe {
            ChartTimeframe::Raw => return Ok(0),
            ChartTimeframe::Hourly => "SELECT aggregate_hourly_charts();",
            ChartTimeframe::Daily => "SELECT aggregate_daily_charts();",
        };
        Ok(diesel::sql_query(query).execute(&mut self.connection)?)
    }

    fn delete_charts(&mut self, timeframe: ChartTimeframe, before: NaiveDateTime) -> Result<usize, DatabaseError> {
        Ok(match timeframe {
            ChartTimeframe::Raw => diesel::delete(charts.filter(raw_created_at.lt(before))).execute(&mut self.connection)?,
            ChartTimeframe::Hourly => diesel::delete(charts_hourly.filter(hourly_created_at.lt(before))).execute(&mut self.connection)?,
            ChartTimeframe::Daily => diesel::delete(charts_daily.filter(daily_created_at.lt(before))).execute(&mut self.connection)?,
        })
    }

    fn get_charts_by_filter(&mut self, filters: Vec<ChartFilter>) -> Result<Vec<(String, f64)>, DatabaseError> {
        let query = filters.into_iter().fold(charts.distinct_on(raw_coin_id).order_by((raw_coin_id, raw_created_at.desc())).into_boxed(), |q, filter| match filter {
            ChartFilter::CreatedBefore(at) => q.filter(raw_created_at.le(at)),
            ChartFilter::CreatedAfter(at) => q.filter(raw_created_at.ge(at)),
            ChartFilter::PriceIds(ids) => q.filter(raw_coin_id.eq_any(ids)),
        });
        Ok(query.select((raw_coin_id, raw_price)).load(&mut self.connection)?)
    }
}

pub(crate) fn chart_extremes(client: &mut DatabaseClient, price_id: &str, timeframe: ChartTimeframe) -> Result<MinMax<f64>, diesel::result::Error> {
    match timeframe {
        ChartTimeframe::Raw => {
            let max = charts
                .filter(raw_coin_id.eq(price_id))
                .filter(raw_price.gt(0.0))
                .order_by(raw_price.desc())
                .select((raw_price, raw_created_at))
                .first::<(f64, NaiveDateTime)>(&mut client.connection)
                .optional()?
                .map(DataPoint::from);
            let min = charts
                .filter(raw_coin_id.eq(price_id))
                .filter(raw_price.gt(0.0))
                .order_by(raw_price.asc())
                .select((raw_price, raw_created_at))
                .first::<(f64, NaiveDateTime)>(&mut client.connection)
                .optional()?
                .map(DataPoint::from);
            Ok(MinMax { max, min })
        }
        ChartTimeframe::Hourly => {
            let max = charts_hourly
                .filter(hourly_coin_id.eq(price_id))
                .filter(hourly_price.gt(0.0))
                .order_by(hourly_price.desc())
                .select((hourly_price, hourly_created_at))
                .first::<(f64, NaiveDateTime)>(&mut client.connection)
                .optional()?
                .map(DataPoint::from);
            let min = charts_hourly
                .filter(hourly_coin_id.eq(price_id))
                .filter(hourly_price.gt(0.0))
                .order_by(hourly_price.asc())
                .select((hourly_price, hourly_created_at))
                .first::<(f64, NaiveDateTime)>(&mut client.connection)
                .optional()?
                .map(DataPoint::from);
            Ok(MinMax { max, min })
        }
        ChartTimeframe::Daily => {
            let max = charts_daily
                .filter(daily_coin_id.eq(price_id))
                .filter(daily_price.gt(0.0))
                .order_by(daily_price.desc())
                .select((daily_price, daily_created_at))
                .first::<(f64, NaiveDateTime)>(&mut client.connection)
                .optional()?
                .map(DataPoint::from);
            let min = charts_daily
                .filter(daily_coin_id.eq(price_id))
                .filter(daily_price.gt(0.0))
                .order_by(daily_price.asc())
                .select((daily_price, daily_created_at))
                .first::<(f64, NaiveDateTime)>(&mut client.connection)
                .optional()?
                .map(DataPoint::from);
            Ok(MinMax { max, min })
        }
    }
}

fn chart_granularity_for_period(period: &ChartPeriod) -> ChartGranularity {
    match period {
        ChartPeriod::Hour => ChartGranularity::Minute,
        ChartPeriod::Day => ChartGranularity::Minute15,
        ChartPeriod::Week => ChartGranularity::Hourly,
        ChartPeriod::Month => ChartGranularity::Hour6,
        ChartPeriod::Year | ChartPeriod::All => ChartGranularity::Daily,
    }
}

fn period_sql(period: ChartPeriod) -> &'static str {
    match period {
        ChartPeriod::Hour => "1 minutes",
        ChartPeriod::Day => "15 minutes",
        ChartPeriod::Week => "1 hour",
        ChartPeriod::Month => "6 hour",
        ChartPeriod::Year => "3 day",
        ChartPeriod::All => "3 day",
    }
}

fn period_minutes(period: ChartPeriod) -> i32 {
    match period {
        ChartPeriod::Hour => 60,
        ChartPeriod::Day => 1440,
        ChartPeriod::Week => 10_080,
        ChartPeriod::Month => 43_200,
        ChartPeriod::Year => 525_600,
        ChartPeriod::All => 10_525_600,
    }
}
