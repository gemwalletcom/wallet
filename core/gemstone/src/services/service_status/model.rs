use std::time::Duration;

use primitives::{Latency, LatencyType};

use crate::formatted_number::GemValueTone;

#[derive(uniffi::Enum, Clone, Debug, PartialEq)]
pub enum GemLatencyStatus {
    Loading,
    Error,
    Result { latency: Latency },
}

#[uniffi::export]
impl GemLatencyStatus {
    pub fn tone(&self) -> GemValueTone {
        match self {
            Self::Loading => GemValueTone::Neutral,
            Self::Error => GemValueTone::Negative,
            Self::Result { latency } => match latency.latency_type {
                LatencyType::Fast => GemValueTone::Positive,
                LatencyType::Normal => GemValueTone::Warning,
                LatencyType::Slow => GemValueTone::Negative,
            },
        }
    }
}

impl From<Option<Duration>> for GemLatencyStatus {
    fn from(duration: Option<Duration>) -> Self {
        match duration {
            Some(duration) => Self::Result {
                latency: Latency::from_milliseconds(duration.as_millis() as u64),
            },
            None => Self::Error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_tones_cover_loading_failure_and_thresholds() {
        assert_eq!(GemLatencyStatus::Loading.tone(), GemValueTone::Neutral);
        assert_eq!(GemLatencyStatus::from(None), GemLatencyStatus::Error);
        assert_eq!(GemLatencyStatus::Error.tone(), GemValueTone::Negative);
        for (milliseconds, tone) in [(1023, GemValueTone::Positive), (1024, GemValueTone::Warning), (2047, GemValueTone::Warning), (2048, GemValueTone::Negative)] {
            assert_eq!(GemLatencyStatus::from(Some(Duration::from_millis(milliseconds))).tone(), tone);
        }
    }
}
