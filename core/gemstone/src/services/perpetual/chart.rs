use std::collections::BTreeMap;
use std::ops::RangeInclusive;
use std::time::Duration;

use primitives::chart::ChartCandleStick;
use primitives::{ChartCandleStickVecExt, DAY, DateTime, HOUR, MINUTE, MONTH, WEEK};
use primitives::{ChartPeriod, Currency, PerpetualDirection, PerpetualPosition, TransactionState};

use super::model::{GemCandleChart, GemCandleMarker, GemCandleTick, GemCandleTickFormat};
use super::rules::chart_layout;
use crate::services::chart::rules::{RIGHT_PADDING, date_style, fraction_of};
use crate::services::chart::zoom::MIN_VISIBLE_POINTS;
use crate::services::chart::{GemChartZoom, candlestick_header};
use crate::services::transactions::model::{GemTransactionRow, GemTransactionTitle};

const CANDLE_BODY_FRACTION: f64 = 0.7;
const LONE_CANDLE_INTERVAL: Duration = MINUTE;
const CANDLE_MARKER_LIMIT: usize = 4;
const X_TICK_COUNT: usize = 4;
const X_TICK_INSET_FRACTION: f64 = 0.1;
const X_TICK_YEAR: Duration = DAY.saturating_mul(360);
const X_TICK_STEPS: [Duration; 23] = [
    MINUTE,
    MINUTE.saturating_mul(2),
    MINUTE.saturating_mul(5),
    MINUTE.saturating_mul(10),
    MINUTE.saturating_mul(15),
    MINUTE.saturating_mul(30),
    HOUR,
    HOUR.saturating_mul(2),
    HOUR.saturating_mul(3),
    HOUR.saturating_mul(4),
    HOUR.saturating_mul(6),
    HOUR.saturating_mul(8),
    HOUR.saturating_mul(12),
    DAY,
    DAY.saturating_mul(2),
    DAY.saturating_mul(3),
    WEEK,
    WEEK.saturating_mul(2),
    WEEK.saturating_mul(4),
    WEEK.saturating_mul(8),
    WEEK.saturating_mul(13),
    WEEK.saturating_mul(26),
    WEEK.saturating_mul(52),
];
const X_TICK_STEP_MONTHS: [u32; 6] = [1, 2, 3, 6, 12, 24];

pub fn candle_chart(
    candles: &[ChartCandleStick],
    period: ChartPeriod,
    price_decimals: usize,
    currency: Currency,
    position: Option<&PerpetualPosition>,
    transactions: &[GemTransactionRow],
    zoom: GemChartZoom,
    utc_offset_seconds: i32,
) -> Option<GemCandleChart> {
    let (first, last) = (candles.first()?, candles.last()?);
    let interval = candles.interval().unwrap_or(LONE_CANDLE_INTERVAL);
    let earliest = first.date.min(last.date - interval * (MIN_VISIBLE_POINTS as u32 - 1));
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
        header: candlestick_header(first.close, last.close, price_decimals, &currency),
        date_style: date_style(period),
        base: first.close,
        price_decimals,
        currency,
        start,
        end,
        body_width: interval.as_secs_f64() * CANDLE_BODY_FRACTION / (end - start).as_secs_f64(),
        x_ticks: x_ticks(visible, start..=end, interval, utc_offset_seconds),
        is_zoomed: zoom.is_zoomed(),
        markers: candle_markers(visible, start..=end, interval, transactions),
    })
}

fn candle_markers(candles: &[ChartCandleStick], window: RangeInclusive<DateTime>, interval: Duration, transactions: &[GemTransactionRow]) -> Vec<GemCandleMarker> {
    let trades: BTreeMap<_, _> = transactions
        .iter()
        .enumerate()
        .filter(|(_, row)| row.state == TransactionState::Confirmed)
        .filter_map(|(position, row)| Some(((row.created_at, position), trade_direction(&row.title)?)))
        .collect();
    let ends = candles.iter().skip(1).map(|next| next.date).chain(candles.last().map(|last| last.date + interval));
    candles
        .iter()
        .zip(ends)
        .filter(|(candle, _)| window.contains(&candle.date))
        .flat_map(|(candle, end)| {
            let directions: Vec<_> = trades.range((candle.date, 0)..(end, 0)).map(|(_, direction)| direction.clone()).collect();
            let hidden = directions.len().saturating_sub(CANDLE_MARKER_LIMIT);
            directions.into_iter().skip(hidden).enumerate().map(move |(level, direction)| GemCandleMarker {
                date: candle.date,
                price: candle.high,
                direction,
                level,
            })
        })
        .collect()
}

fn trade_direction(title: &GemTransactionTitle) -> Option<PerpetualDirection> {
    match title {
        GemTransactionTitle::PerpetualOpen { direction } => direction.clone(),
        GemTransactionTitle::PerpetualClose { direction: Some(PerpetualDirection::Long) } => Some(PerpetualDirection::Short),
        GemTransactionTitle::PerpetualClose { direction: Some(PerpetualDirection::Short) } => Some(PerpetualDirection::Long),
        _ => None,
    }
}

fn x_ticks(candles: &[ChartCandleStick], window: RangeInclusive<DateTime>, interval: Duration, utc_offset_seconds: i32) -> Vec<GemCandleTick> {
    let (start, end) = window.into_inner();
    let label_range = (start + fraction_of(end - start, X_TICK_INSET_FRACTION))..=(end - interval / 2);
    let (candles_per_step, step) = tick_step(*label_range.end() - *label_range.start(), interval);
    let previous = std::iter::once(None).chain(candles.iter().map(Some));
    let dates: Vec<DateTime> = candles
        .iter()
        .zip(previous)
        .filter(|(candle, previous)| {
            let number = candle_number(candle.date, utc_offset_seconds, interval);
            let previous_number = previous.map_or(number - 1, |previous| candle_number(previous.date, utc_offset_seconds, interval));
            label_range.contains(&candle.date) && previous_number.div_euclid(i64::from(candles_per_step)) != number.div_euclid(i64::from(candles_per_step))
        })
        .map(|(candle, _)| candle.date)
        .collect();
    let format = tick_format(end - start, step);
    let is_multi_day = match candles {
        [first, .., last] => last.date - first.date > DAY,
        _ => false,
    };
    dates
        .iter()
        .map(|date| GemCandleTick {
            date: *date,
            format: match format {
                GemCandleTickFormat::Time if is_multi_day && is_day_start(*date, &dates, utc_offset_seconds) => GemCandleTickFormat::Day,
                format => format,
            },
        })
        .collect()
}

fn tick_step(span: Duration, interval: Duration) -> (u32, Duration) {
    let (unit, ladder): (Duration, Vec<u32>) = match is_monthly(interval) {
        true => (MONTH, X_TICK_STEP_MONTHS.to_vec()),
        false => (
            interval,
            X_TICK_STEPS
                .into_iter()
                .filter(|step| step.as_secs() % interval.as_secs() == 0)
                .map(|step| (step.as_secs() / interval.as_secs()) as u32)
                .collect(),
        ),
    };
    let fits = |count: &u32| unit * (*count * X_TICK_COUNT as u32) > span;
    let candles_per_step = ladder.into_iter().find(fits).unwrap_or((span.as_secs() / (unit.as_secs() * X_TICK_COUNT as u64)) as u32 + 1);
    (candles_per_step, unit * candles_per_step)
}

fn candle_number(date: DateTime, utc_offset_seconds: i32, interval: Duration) -> i64 {
    let local = date.to_local(utc_offset_seconds);
    match is_monthly(interval) {
        true => {
            let (year, month, _) = local.date();
            i64::from(year) * 12 + i64::from(month - 1)
        }
        false => local.timestamp().div_euclid(interval.as_secs() as i64),
    }
}

fn is_monthly(interval: Duration) -> bool {
    interval > WEEK
}

fn tick_format(span: Duration, step: Duration) -> GemCandleTickFormat {
    if span >= X_TICK_YEAR {
        GemCandleTickFormat::MonthYear
    } else if step >= DAY {
        GemCandleTickFormat::Day
    } else {
        GemCandleTickFormat::Time
    }
}

fn is_day_start(date: DateTime, ticks: &[DateTime], utc_offset_seconds: i32) -> bool {
    let local = |date: DateTime| date.to_local(utc_offset_seconds);
    let is_earlier_day = ticks.last().is_some_and(|latest| local(*latest).date() != local(date).date());
    let is_first_of_its_day = ticks.iter().find(|tick| local(**tick).date() == local(date).date()) == Some(&date);
    (is_earlier_day && is_first_of_its_day) || local(date) == local(date).truncated(DAY)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::DurationExt;

    #[test]
    fn test_candle_chart() {
        let at = DateTime::from_timestamp;
        let candles = ChartCandleStick::mock_series(DateTime::UNIX_EPOCH, MINUTE, 40);
        let chart = |candles: &[ChartCandleStick], scale: f64, offset: f64| candle_chart(candles, ChartPeriod::Hour, 2, Currency::USD, None, &[], GemChartZoom { scale, offset }, 0).unwrap();
        let whole = chart(&candles, 1.0, 0.0);
        let zoomed = chart(&candles, 2.0, 0.0);
        let panned = chart(&candles, 2.0, 0.25);
        let lone = chart(&candles[..1], 1.0, 0.0);

        assert_eq!((whole.start, whole.end, whole.body_width), (at(-30), DateTime::from_timestamp_millis(2_386_800), 60_000.0 * 0.7 / 2_416_800.0));
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
            Some(candlestick_header(candles[0].close, candles[9].close, 2, &Currency::USD)),
            "a selection is measured from the period's first close"
        );
        assert_eq!((lone.start, lone.end, lone.body_width), (at(-810), at(30), 0.05), "a lone candle is drawn at the fourteen-candle width");
        assert_eq!(candle_chart(&[], ChartPeriod::Hour, 2, Currency::USD, None, &[], GemChartZoom::default(), 0), None);
    }

    #[test]
    fn test_candle_markers() {
        use crate::services::transactions::rules::row;
        use primitives::PerpetualDirection::{Long, Short};
        use primitives::TransactionType::{PerpetualClosePosition as Close, PerpetualModifyPosition as Modify, PerpetualOpenPosition as Open};
        use primitives::{Transaction, TransactionListItem, TransactionPerpetualMetadata, TransactionType};
        let trade = |transaction_type: TransactionType, direction, seconds| Transaction {
            transaction_type,
            created_at: DateTime::from_timestamp(seconds),
            metadata: serde_json::to_value(TransactionPerpetualMetadata {
                direction,
                ..TransactionPerpetualMetadata::mock()
            })
            .ok(),
            ..Transaction::mock()
        };
        let markers = |candles: &[ChartCandleStick], zoom: GemChartZoom, trades: &[Transaction]| {
            let rows: Vec<GemTransactionRow> = trades.iter().map(|trade| row(&TransactionListItem::mock_transaction(trade.clone()))).collect();
            let chart = candle_chart(candles, ChartPeriod::Hour, 2, Currency::USD, None, &rows, zoom, 0).unwrap();
            chart.markers.into_iter().map(|marker| (marker.date.timestamp(), marker.direction, marker.level, marker.price)).collect::<Vec<_>>()
        };
        let minutes = ChartCandleStick::mock_series(DateTime::UNIX_EPOCH, MINUTE, 40);
        let months: Vec<ChartCandleStick> = (0..3).map(|month| ChartCandleStick::mock(DateTime::from_date(1970, month + 1, 1).unwrap().timestamp(), 100.0)).collect();
        let pending = Transaction {
            state: TransactionState::Pending,
            ..trade(Open, Long, 125)
        };

        assert_eq!(
            markers(
                &minutes,
                GemChartZoom::default(),
                &[trade(Close, Long, 170), trade(Open, Short, 150), trade(Close, Short, 140), trade(Open, Long, 125), trade(Open, Short, 121)]
            ),
            vec![(120, Long, 0, 103.0), (120, Long, 1, 103.0), (120, Short, 2, 103.0), (120, Short, 3, 103.0)],
            "a candle stacks its latest four trades over its high, the oldest lowest, a close marked as the side it trades"
        );
        assert_eq!(
            markers(&minutes, GemChartZoom::default(), &[pending, trade(Modify, Long, 125), trade(Open, Long, 2400)]),
            vec![],
            "only filled trades on the drawn candles are marked"
        );
        assert_eq!(
            markers(&months, GemChartZoom::default(), &[trade(Open, Long, 29 * 86_400)]),
            vec![(0, Long, 0, 101.0)],
            "a trade late in a long month stays on its candle"
        );
        assert_eq!(
            markers(&minutes, GemChartZoom { scale: 2.0, offset: 0.25 }, &[trade(Open, Long, 545), trade(Open, Short, 610), trade(Open, Long, 1805)]),
            vec![(600, Short, 0, 111.0)],
            "a candle whose middle is off the plot carries no marker"
        );
    }

    #[test]
    fn test_x_ticks() {
        use GemCandleTickFormat::{Day, MonthYear, Time};
        let at = |date: &str| DateTime::parse_rfc3339(date).unwrap();
        let axis = |candles: &[ChartCandleStick], zoom: GemChartZoom, utc_offset_seconds: i32| -> Vec<(DateTime, GemCandleTickFormat)> {
            candle_chart(candles, ChartPeriod::Day, 2, Currency::USD, None, &[], zoom, utc_offset_seconds)
                .unwrap()
                .x_ticks
                .into_iter()
                .map(|tick| (tick.date, tick.format))
                .collect()
        };
        let monthly: Vec<ChartCandleStick> = (0..14)
            .map(|month| ChartCandleStick::mock(DateTime::from_date(2025 + (7 + month as i32) / 12, (7 + month) % 12 + 1, 7).unwrap().timestamp(), 1.0))
            .collect();
        let whole = GemChartZoom::default();

        assert_eq!(
            axis(&ChartCandleStick::mock_series(at("2026-09-23T15:08:00Z"), MINUTE, 61), whole, 0),
            vec![(at("2026-09-23T15:15:00Z"), Time), (at("2026-09-23T15:30:00Z"), Time), (at("2026-09-23T15:45:00Z"), Time), (at("2026-09-23T16:00:00Z"), Time)]
        );
        assert_eq!(
            axis(&ChartCandleStick::mock_series(at("2026-09-22T16:30:00Z"), Duration::from_mins(30), 48), whole, 3 * 3_600),
            vec![(at("2026-09-22T21:00:00Z"), Time), (at("2026-09-23T03:00:00Z"), Time), (at("2026-09-23T09:00:00Z"), Time), (at("2026-09-23T15:00:00Z"), Time)],
            "the marks are on the user's own clock"
        );
        assert_eq!(
            axis(&ChartCandleStick::mock_series(at("2026-09-22T04:00:00Z"), Duration::from_hours(2), 17), whole, 0),
            vec![(at("2026-09-22T08:00:00Z"), Day), (at("2026-09-22T16:00:00Z"), Time), (at("2026-09-23T00:00:00Z"), Day), (at("2026-09-23T08:00:00Z"), Time)],
            "the first label of an earlier day and a label at midnight name the day"
        );
        assert_eq!(axis(&monthly, whole, 0), vec![(at("2026-01-07T00:00:00Z"), MonthYear), (at("2026-07-07T00:00:00Z"), MonthYear)]);
        assert_eq!(
            axis(&ChartCandleStick::mock_series(at("2025-11-17T00:00:00Z"), Duration::days(8), 46), whole, 0),
            vec![
                (at("2026-01-04T00:00:00Z"), MonthYear),
                (at("2026-04-02T00:00:00Z"), MonthYear),
                (at("2026-07-07T00:00:00Z"), MonthYear),
                (at("2026-10-03T00:00:00Z"), MonthYear)
            ],
            "candles shorter than a month label only the first one of each month"
        );
        assert_eq!(
            axis(&ChartCandleStick::mock_series(at("2013-03-28T00:00:00Z"), Duration::days(84), 60), whole, 0),
            vec![
                (at("2017-05-18T00:00:00Z"), MonthYear),
                (at("2020-05-14T00:00:00Z"), MonthYear),
                (at("2023-08-03T00:00:00Z"), MonthYear),
                (at("2026-10-22T00:00:00Z"), MonthYear)
            ],
            "candles longer than a month label the first one of each step, even when no candle starts in the step's first month"
        );
        assert_eq!(
            axis(&ChartCandleStick::mock_series(at("2026-09-23T15:08:00Z"), MINUTE, 5), whole, 0),
            vec![(at("2026-09-23T15:10:00Z"), Time)],
            "the first candle on screen is labelled only when it sits on the step grid"
        );
        assert!(
            [0.25, 0.26, 0.27, 0.28]
                .into_iter()
                .all(|offset| axis(&ChartCandleStick::mock_series(at("2026-09-23T15:00:00Z"), MINUTE, 40), GemChartZoom { scale: 2.0, offset }, 0)
                    .iter()
                    .all(|(date, _)| date.timestamp() % 300 == 0)),
            "a pan keeps the step, so the labels slide with their candles"
        );
        for (interval, count) in [
            (MINUTE, 60),
            (Duration::from_mins(30), 48),
            (Duration::from_hours(4), 42),
            (Duration::from_hours(12), 60),
            (WEEK, 260),
            (Duration::days(30), 130),
            (Duration::days(84), 60),
        ] {
            let labels = axis(&ChartCandleStick::mock_series(at("2026-01-05T00:00:00Z"), interval, count), whole, 0).len();
            assert!((2..=X_TICK_COUNT).contains(&labels), "{count} candles of {interval:?} have {labels} labels");
        }
    }
}
