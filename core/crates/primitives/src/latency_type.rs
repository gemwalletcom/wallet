use serde::{Deserialize, Serialize};
use typeshare::typeshare;

const FAST_LATENCY_MILLISECONDS: u64 = 1024;
const NORMAL_LATENCY_MILLISECONDS: u64 = 2048;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum LatencyType {
    Fast,
    Normal,
    Slow,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Latency {
    pub latency_type: LatencyType,
    pub value: f64,
}

impl Latency {
    pub fn from_milliseconds(milliseconds: u64) -> Self {
        let latency_type = match milliseconds {
            value if value < FAST_LATENCY_MILLISECONDS => LatencyType::Fast,
            value if value < NORMAL_LATENCY_MILLISECONDS => LatencyType::Normal,
            _ => LatencyType::Slow,
        };
        Self {
            latency_type,
            value: milliseconds as f64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_milliseconds() {
        assert_eq!(Latency::from_milliseconds(0).latency_type, LatencyType::Fast);
        assert_eq!(Latency::from_milliseconds(1023).latency_type, LatencyType::Fast);
        assert_eq!(Latency::from_milliseconds(1024).latency_type, LatencyType::Normal);
        assert_eq!(Latency::from_milliseconds(2047).latency_type, LatencyType::Normal);
        assert_eq!(Latency::from_milliseconds(2048).latency_type, LatencyType::Slow);
        assert_eq!(Latency::from_milliseconds(440).value, 440.0);
    }
}
