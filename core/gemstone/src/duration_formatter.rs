#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemDurationUnit {
    Day,
    Hour,
    Minute,
    Second,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemDurationPart {
    pub value: i64,
    pub unit: GemDurationUnit,
}

#[uniffi::export]
pub fn estimated_duration_parts(seconds: i64) -> Option<Vec<GemDurationPart>> {
    estimate_parts(seconds)
}

#[uniffi::export]
pub fn estimated_duration_text(duration: String) -> String {
    match duration.is_empty() {
        true => duration,
        false => format!("≈ {duration}"),
    }
}

const MINUTE_SECONDS: i64 = 60;
const HOUR_SECONDS: i64 = 60 * MINUTE_SECONDS;
const DAY_SECONDS: i64 = 24 * HOUR_SECONDS;

pub(crate) fn day_parts(seconds: i64) -> Option<Vec<GemDurationPart>> {
    match seconds / DAY_SECONDS {
        days if days > 0 => Some(vec![part(days, GemDurationUnit::Day)]),
        _ => None,
    }
}

pub(crate) fn countdown_parts(seconds: i64) -> Vec<GemDurationPart> {
    if seconds < 0 {
        return vec![];
    }
    let parts = if seconds < DAY_SECONDS {
        [part(seconds / HOUR_SECONDS, GemDurationUnit::Hour), part(seconds % HOUR_SECONDS / MINUTE_SECONDS, GemDurationUnit::Minute)]
    } else {
        [part(seconds / DAY_SECONDS, GemDurationUnit::Day), part(seconds % DAY_SECONDS / HOUR_SECONDS, GemDurationUnit::Hour)]
    };
    let kept: Vec<GemDurationPart> = parts.iter().skip_while(|part| part.value == 0).cloned().collect();
    match kept.is_empty() {
        true => parts.last().cloned().into_iter().collect(),
        false => kept,
    }
}

fn estimate_parts(seconds: i64) -> Option<Vec<GemDurationPart>> {
    if seconds <= 0 {
        return None;
    }
    Some(
        [part(seconds / MINUTE_SECONDS, GemDurationUnit::Minute), part(seconds % MINUTE_SECONDS, GemDurationUnit::Second)]
            .into_iter()
            .filter(|part| part.value > 0)
            .collect(),
    )
}

fn part(value: i64, unit: GemDurationUnit) -> GemDurationPart {
    GemDurationPart { value, unit }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_countdown_drops_to_hours_and_minutes_inside_a_day() {
        assert_eq!(countdown_parts(2 * DAY_SECONDS + 3 * HOUR_SECONDS), vec![part(2, GemDurationUnit::Day), part(3, GemDurationUnit::Hour)]);
        assert_eq!(countdown_parts(3 * HOUR_SECONDS + 4 * MINUTE_SECONDS), vec![part(3, GemDurationUnit::Hour), part(4, GemDurationUnit::Minute)]);
        assert_eq!(countdown_parts(4 * MINUTE_SECONDS), vec![part(4, GemDurationUnit::Minute)], "a leading zero is dropped");
        assert_eq!(countdown_parts(0), vec![part(0, GemDurationUnit::Minute)], "nothing left still reads as zero minutes");
        assert_eq!(countdown_parts(-1), vec![]);
    }

    #[test]
    fn test_an_estimated_duration_reads_as_approximate_and_no_duration_stays_empty() {
        assert_eq!(estimated_duration_text("12 min".to_string()), "≈ 12 min");
        assert_eq!(estimated_duration_text(String::new()), "");
    }

    #[test]
    fn test_an_estimate_reads_in_minutes_and_seconds() {
        assert_eq!(estimate_parts(90), Some(vec![part(1, GemDurationUnit::Minute), part(30, GemDurationUnit::Second)]));
        assert_eq!(estimate_parts(45), Some(vec![part(45, GemDurationUnit::Second)]));
        assert_eq!(estimate_parts(720), Some(vec![part(12, GemDurationUnit::Minute)]));
        assert_eq!(estimate_parts(0), None, "no estimate reads as nothing, never as an empty duration");
    }

    #[test]
    fn test_days_read_only_once_a_day_is_reached() {
        assert_eq!(day_parts(2 * DAY_SECONDS), Some(vec![part(2, GemDurationUnit::Day)]));
        assert_eq!(day_parts(DAY_SECONDS - 1), None);
    }
}
