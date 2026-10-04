use crate::{Feature, FeaturePolicy};

impl FeaturePolicy {
    pub fn mock() -> Self {
        Self {
            feature: Feature::Buy,
            country_code: "FR".to_string(),
            is_enabled: false,
        }
    }
}
