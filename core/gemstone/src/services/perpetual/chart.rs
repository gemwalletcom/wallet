use std::ops::RangeInclusive;

use chrono::{DateTime, Datelike, NaiveTime, TimeDelta, Utc};
use primitives::chart::ChartCandleStick;
use primitives::{ChartPeriod, PerpetualPosition};

use super::model::{GemCandleChart, GemCandleTick, GemCandleTickFormat};
use super::rules::chart_layout;
use crate::services::chart::rules::{RIGHT_PADDING, date_style, fraction_of};
use crate::services::chart::zoom::MIN_VISIBLE_POINTS;
use crate::services::chart::{GemChartZoom, candlestick_header};

const CANDLE_BODY_FRACTION: f64 = 0.7;
const LONE_CANDLE_INTERVAL: TimeDelta = TimeDelta::minutes(1);
const X_TICK_COUNT: usize = 4;
const X_TICK_INSET_FRACTION: f64 = 0.1;
const X_TICK_YEAR: TimeDelta = TimeDelta::days(360);
const X_TICK_STEPS: [TimeDelta; 23] = [
    TimeDelta::minutes(1),
    TimeDelta::minutes(2),
    TimeDelta::minutes(5),
    TimeDelta::minutes(10),
    TimeDelta::minutes(15),
    TimeDelta::minutes(30),
    TimeDelta::hours(1),
    TimeDelta::hours(2),
    TimeDelta::hours(3),
    TimeDelta::hours(4),
    TimeDelta::hours(6),
    TimeDelta::hours(8),
    TimeDelta::hours(12),
    TimeDelta::days(1),
    TimeDelta::days(2),
    TimeDelta::days(3),
    TimeDelta::weeks(1),
    TimeDelta::weeks(2),
    TimeDelta::weeks(4),
    TimeDelta::weeks(8),
    TimeDelta::weeks(13),
    TimeDelta::weeks(26),
    TimeDelta::weeks(52),
];
const X_TICK_STEP_MONTHS: [i64; 6] = [1, 2, 3, 6, 12, 24];
const X_TICK_MONTH: TimeDelta = TimeDelta::days(30);

pub fn candle_chart(candles: &[ChartCandleStick], period: ChartPeriod, price_decimals: u32, position: Option<&PerpetualPosition>, zoom: GemChartZoom, utc_offset: TimeDelta) -> Option<GemCandleChart> {
    let (first, last) = (candles.first()?, candles.last()?);
    let interval = candles
        .iter()
        .zip(&candles[1..])
        .map(|(previous, next)| next.date - previous.date)
        .filter(|gap| *gap > TimeDelta::zero())
        .min()
        .unwrap_or(LONE_CANDLE_INTERVAL);
    let earliest = first.date.min(last.date - interval * (MIN_VISIBLE_POINTS as i32 - 1));
    let zoom = zoom.clamped(candles.len());
    let (window_start, window_end) = zoom.window(earliest, last.date).into_inner();
    let start = window_start - interval / 2;
    let end = window_end + fraction_of(window_end - window_start, RIGHT_PADDING).max(interval / 2);
    let from = candles.partition_point(|candle| candle.date <= window_start - interval);
    let to = candles.partition_point(|candle| candle.date < end + interval / 2).max(from + 1);
    let visible = &candles[from..to];
    Some(GemCandleChart {
        candles: visible.to_vec(),
        layout: chart_layout(visible, last, position, price_decimals),
        header: candlestick_header(first.close, last.close, price_decimals),
        date_style: date_style(period),
        base: first.close,
        price_decimals,
        start,
        end,
        body_width: interval.num_milliseconds() as f64 * CANDLE_BODY_FRACTION / (end - start).num_milliseconds() as f64,
        x_ticks: x_ticks(visible, start..=end, interval, utc_offset),
        is_zoomed: zoom.is_zoomed(),
    })
}

fn x_ticks(candles: &[ChartCandleStick], window: RangeInclusive<DateTime<Utc>>, interval: TimeDelta, utc_offset: TimeDelta) -> Vec<GemCandleTick> {
    let (start, end) = window.into_inner();
    let label_range = (start + fraction_of(end - start, X_TICK_INSET_FRACTION))..=(end - interval / 2);
    let (candles_per_step, step) = tick_step(*label_range.end() - *label_range.start(), interval);
    let dates: Vec<DateTime<Utc>> = candles
        .iter()
        .map(|candle| candle.date)
        .filter(|date| label_range.contains(date) && candle_number(*date, utc_offset, interval).rem_euclid(candles_per_step) == 0)
        .collect();
    let format = tick_format(end - start, step);
    let is_multi_day = match candles {
        [first, .., last] => last.date - first.date > TimeDelta::days(1),
        _ => false,
    };
    dates
        .iter()
        .map(|date| GemCandleTick {
            date: *date,
            format: match format {
                GemCandleTickFormat::Time if is_multi_day && is_day_start(*date, &dates, utc_offset) => GemCandleTickFormat::Day,
                format => format,
            },
        })
        .collect()
}

fn tick_step(span: TimeDelta, interval: TimeDelta) -> (i64, TimeDelta) {
    let (unit, ladder): (TimeDelta, Vec<i64>) = match is_monthly(interval) {
        true => (X_TICK_MONTH, X_TICK_STEP_MONTHS.to_vec()),
        false => (
            interval,
            X_TICK_STEPS
                .into_iter()
                .filter(|step| step.num_seconds() % interval.num_seconds() == 0)
                .map(|step| step.num_seconds() / interval.num_seconds())
                .collect(),
        ),
    };
    let fits = |count: &i64| unit * (*count * X_TICK_COUNT as i64) as i32 > span;
    let candles_per_step = ladder.into_iter().find(fits).unwrap_or(span.num_seconds() / (unit.num_seconds() * X_TICK_COUNT as i64) + 1);
    (candles_per_step, unit * candles_per_step as i32)
}

fn candle_number(date: DateTime<Utc>, utc_offset: TimeDelta, interval: TimeDelta) -> i64 {
    let local = date + utc_offset;
    match is_monthly(interval) {
        true => i64::from(local.year()) * 12 + i64::from(local.month0()),
        false => local.timestamp().div_euclid(interval.num_seconds()),
    }
}

fn is_monthly(interval: TimeDelta) -> bool {
    interval > TimeDelta::weeks(1)
}

fn tick_format(span: TimeDelta, step: TimeDelta) -> GemCandleTickFormat {
    if span >= X_TICK_YEAR {
        GemCandleTickFormat::MonthYear
    } else if step >= TimeDelta::days(1) {
        GemCandleTickFormat::Day
    } else {
        GemCandleTickFormat::Time
    }
}

fn is_day_start(date: DateTime<Utc>, ticks: &[DateTime<Utc>], utc_offset: TimeDelta) -> bool {
    let local = |date: DateTime<Utc>| (date + utc_offset).naive_utc();
    let is_earlier_day = ticks.last().is_some_and(|latest| local(*latest).date() != local(date).date());
    let is_first_of_its_day = ticks.iter().find(|tick| local(**tick).date() == local(date).date()) == Some(&date);
    (is_earlier_day && is_first_of_its_day) || local(date).time() == NaiveTime::MIN
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candle_chart() {
        let at = |seconds: i64| DateTime::from_timestamp(seconds, 0).unwrap();
        let candles = ChartCandleStick::mock_series(DateTime::UNIX_EPOCH, TimeDelta::minutes(1), 40);
        let chart = |candles: &[ChartCandleStick], scale: f64, offset: f64| candle_chart(candles, ChartPeriod::Hour, 2, None, GemChartZoom { scale, offset }, TimeDelta::zero()).unwrap();
        let whole = chart(&candles, 1.0, 0.0);
        let zoomed = chart(&candles, 2.0, 0.0);
        let panned = chart(&candles, 2.0, 0.25);
        let lone = chart(&candles[..1], 1.0, 0.0);

        assert_eq!((whole.start, whole.end, whole.body_width), (at(-30), DateTime::from_timestamp_millis(2_386_800).unwrap(), 60_000.0 * 0.7 / 2_416_800.0));
        assert_eq!((zoomed.start, zoomed.end, zoomed.candles.len()), (at(1140), at(2370), 21), "the candle straddling the left edge stays drawn");
        assert_eq!(zoomed.layout, chart_layout(&candles[19..], &candles[39], None, 2), "the price axis fits the candles on screen");
        assert_eq!(chart(&candles, 100.0, 0.0).candles, candles[25..], "a pinch stops with fourteen candles on screen");
        assert_eq!(
            (whole.is_zoomed, zoomed.is_zoomed, chart(&candles[..10], 2.0, 0.0).is_zoomed),
            (false, true, false),
            "a market with fewer than fourteen candles never zooms"
        );
        assert_eq!((panned.start, panned.end), (at(555), at(1785)));
        assert_eq!(panned.index_at(0.0), Some(1), "a candle whose middle is off the plot is never picked");
        assert_eq!(
            panned.selection(0).map(|selection| selection.header),
            Some(candlestick_header(candles[0].close, candles[9].close, 2)),
            "a selection is measured from the period's first close"
        );
        assert_eq!((lone.start, lone.end, lone.body_width), (at(-810), at(30), 0.05), "a lone candle is drawn at the fourteen-candle width");
        assert_eq!(candle_chart(&[], ChartPeriod::Hour, 2, None, GemChartZoom::default(), TimeDelta::zero()), None);
    }

    #[test]
    fn test_x_ticks() {
        use GemCandleTickFormat::{Day, MonthYear, Time};
        let at = |date: &str| DateTime::parse_from_rfc3339(date).unwrap().to_utc();
        let axis = |candles: &[ChartCandleStick], zoom: GemChartZoom, utc_offset: TimeDelta| -> Vec<(DateTime<Utc>, GemCandleTickFormat)> {
            candle_chart(candles, ChartPeriod::Day, 2, None, zoom, utc_offset).unwrap().x_ticks.into_iter().map(|tick| (tick.date, tick.format)).collect()
        };
        let monthly: Vec<ChartCandleStick> = (0..14)
            .map(|month| ChartCandleStick::mock(at("2025-08-07T00:00:00Z").checked_add_months(chrono::Months::new(month)).unwrap().timestamp(), 1.0))
            .collect();
        let whole = GemChartZoom::default();

        assert_eq!(
            axis(&ChartCandleStick::mock_series(at("2026-09-23T15:08:00Z"), TimeDelta::minutes(1), 61), whole, TimeDelta::zero()),
            vec![(at("2026-09-23T15:15:00Z"), Time), (at("2026-09-23T15:30:00Z"), Time), (at("2026-09-23T15:45:00Z"), Time), (at("2026-09-23T16:00:00Z"), Time)]
        );
        assert_eq!(
            axis(&ChartCandleStick::mock_series(at("2026-09-22T16:30:00Z"), TimeDelta::minutes(30), 48), whole, TimeDelta::hours(3)),
            vec![(at("2026-09-22T21:00:00Z"), Time), (at("2026-09-23T03:00:00Z"), Time), (at("2026-09-23T09:00:00Z"), Time), (at("2026-09-23T15:00:00Z"), Time)],
            "the marks are on the user's own clock"
        );
        assert_eq!(
            axis(&ChartCandleStick::mock_series(at("2026-09-22T04:00:00Z"), TimeDelta::hours(2), 17), whole, TimeDelta::zero()),
            vec![(at("2026-09-22T08:00:00Z"), Day), (at("2026-09-22T16:00:00Z"), Time), (at("2026-09-23T00:00:00Z"), Day), (at("2026-09-23T08:00:00Z"), Time)],
            "the first label of an earlier day and a label at midnight name the day"
        );
        assert_eq!(axis(&monthly, whole, TimeDelta::zero()), vec![(at("2026-01-07T00:00:00Z"), MonthYear), (at("2026-07-07T00:00:00Z"), MonthYear)]);
        assert!(
            [0.25, 0.26, 0.27, 0.28].into_iter().all(
                |offset| axis(&ChartCandleStick::mock_series(at("2026-09-23T15:00:00Z"), TimeDelta::minutes(1), 40), GemChartZoom { scale: 2.0, offset }, TimeDelta::zero())
                    .iter()
                    .all(|(date, _)| date.timestamp() % 300 == 0)
            ),
            "a pan keeps the step, so the labels slide with their candles"
        );
        for (interval, count) in [
            (TimeDelta::minutes(1), 60),
            (TimeDelta::minutes(30), 48),
            (TimeDelta::hours(4), 42),
            (TimeDelta::hours(12), 60),
            (TimeDelta::weeks(1), 260),
            (TimeDelta::days(30), 130),
        ] {
            let labels = axis(&ChartCandleStick::mock_series(at("2026-01-05T00:00:00Z"), interval, count), whole, TimeDelta::zero()).len();
            assert!((2..=X_TICK_COUNT).contains(&labels), "{count} candles of {interval} have {labels} labels");
        }
    }
}
