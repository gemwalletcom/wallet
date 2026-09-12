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

#[derive(Default, uniffi::Object)]
pub struct DurationFormatter {}

#[uniffi::export]
impl DurationFormatter {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn countdown_parts(&self, seconds: i64) -> Vec<GemDurationPart> {
        countdown_parts(seconds)
    }

    pub fn estimate_parts(&self, seconds: i64) -> Vec<GemDurationPart> {
        estimate_parts(seconds)
    }
}

const MINUTE_SECONDS: i64 = 60;
const HOUR_SECONDS: i64 = 60 * MINUTE_SECONDS;
const DAY_SECONDS: i64 = 24 * HOUR_SECONDS;

fn countdown_parts(seconds: i64) -> Vec<GemDurationPart> {
    if seconds < 0 {
        return vec![];
    }
    let parts = if seconds < DAY_SECONDS {
        [
            part(seconds / HOUR_SECONDS, GemDurationUnit::Hour),
            part(seconds % HOUR_SECONDS / MINUTE_SECONDS, GemDurationUnit::Minute),
        ]
    } else {
        [
            part(seconds / DAY_SECONDS, GemDurationUnit::Day),
            part(seconds % DAY_SECONDS / HOUR_SECONDS, GemDurationUnit::Hour),
        ]
    };
    let kept: Vec<GemDurationPart> = parts.iter().skip_while(|part| part.value == 0).cloned().collect();
    match kept.is_empty() {
        true => parts.last().cloned().into_iter().collect(),
        false => kept,
    }
}

fn estimate_parts(seconds: i64) -> Vec<GemDurationPart> {
    if seconds <= 0 {
        return vec![];
    }
    [
        part(seconds / MINUTE_SECONDS, GemDurationUnit::Minute),
        part(seconds % MINUTE_SECONDS, GemDurationUnit::Second),
    ]
    .into_iter()
    .filter(|part| part.value > 0)
    .collect()
}

fn part(value: i64, unit: GemDurationUnit) -> GemDurationPart {
    GemDurationPart { value, unit }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_countdown_drops_to_hours_and_minutes_inside_a_day() {
        assert_eq!(
            countdown_parts(2 * DAY_SECONDS + 3 * HOUR_SECONDS),
            vec![part(2, GemDurationUnit::Day), part(3, GemDurationUnit::Hour)]
        );
        assert_eq!(
            countdown_parts(3 * HOUR_SECONDS + 4 * MINUTE_SECONDS),
            vec![part(3, GemDurationUnit::Hour), part(4, GemDurationUnit::Minute)]
        );
        assert_eq!(countdown_parts(4 * MINUTE_SECONDS), vec![part(4, GemDurationUnit::Minute)], "a leading zero is dropped");
        assert_eq!(countdown_parts(0), vec![part(0, GemDurationUnit::Minute)], "nothing left still reads as zero minutes");
        assert_eq!(countdown_parts(-1), vec![]);
    }

    #[test]
    fn test_an_estimate_reads_in_minutes_and_seconds() {
        assert_eq!(
            estimate_parts(90),
            vec![part(1, GemDurationUnit::Minute), part(30, GemDurationUnit::Second)]
        );
        assert_eq!(estimate_parts(45), vec![part(45, GemDurationUnit::Second)]);
        assert_eq!(estimate_parts(720), vec![part(12, GemDurationUnit::Minute)]);
        assert_eq!(estimate_parts(0), vec![]);
    }
}
