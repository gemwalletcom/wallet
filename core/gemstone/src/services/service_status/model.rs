use std::collections::HashMap;
use std::time::Duration;

use primitives::{Latency, LatencyType};

use super::rules;
use crate::formatted_number::GemValueTone;
use crate::models::list::GemListSection;

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

#[derive(uniffi::Enum, Clone, Debug, PartialEq)]
pub enum GemServiceStatusTarget {
    Endpoint { host: String },
    Stream,
}

#[derive(uniffi::Record, Clone, Debug, PartialEq)]
pub struct GemServiceStatusSession {
    pub endpoints: HashMap<String, GemLatencyStatus>,
    pub stream: GemLatencyStatus,
}

impl Default for GemServiceStatusSession {
    fn default() -> Self {
        Self {
            endpoints: HashMap::new(),
            stream: GemLatencyStatus::Loading,
        }
    }
}

#[uniffi::export]
impl GemServiceStatusSession {
    pub fn targets(&self) -> Vec<GemServiceStatusTarget> {
        rules::targets()
    }

    pub fn on_status(&self, target: GemServiceStatusTarget, status: GemLatencyStatus) -> Self {
        match target {
            GemServiceStatusTarget::Endpoint { host } => {
                let mut endpoints = self.endpoints.clone();
                endpoints.insert(host, status);
                Self { endpoints, ..self.clone() }
            }
            GemServiceStatusTarget::Stream => Self { stream: status, ..self.clone() },
        }
    }

    pub fn sections(&self) -> Vec<GemListSection> {
        rules::sections(&self.endpoints, self.stream.clone())
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
