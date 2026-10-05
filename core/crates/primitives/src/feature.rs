use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsRefStr, EnumString)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum Feature {
    Buy,
    Sell,
    Swap,
    Perpetuals,
    Rewards,
    Staking,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Features {
    pub buy: bool,
    pub sell: bool,
    pub swap: bool,
    pub perpetuals: bool,
    pub rewards: bool,
    pub staking: bool,
}

impl Features {
    pub fn is_enabled(&self, feature: Feature) -> bool {
        match feature {
            Feature::Buy => self.buy,
            Feature::Sell => self.sell,
            Feature::Swap => self.swap,
            Feature::Perpetuals => self.perpetuals,
            Feature::Rewards => self.rewards,
            Feature::Staking => self.staking,
        }
    }

    pub fn for_country(policies: &[FeaturePolicy], country_code: &str) -> Self {
        let enabled = |feature| policies.iter().filter(|policy| policy.feature == feature && policy.country_code == country_code).all(|policy| policy.is_enabled);
        Self {
            buy: enabled(Feature::Buy),
            sell: enabled(Feature::Sell),
            swap: enabled(Feature::Swap),
            perpetuals: enabled(Feature::Perpetuals),
            rewards: enabled(Feature::Rewards),
            staking: enabled(Feature::Staking),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeaturePolicy {
    pub feature: Feature,
    pub country_code: String,
    pub is_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::{Feature, FeaturePolicy, Features};

    #[test]
    fn test_for_country() {
        assert_eq!(
            Features::for_country(&[], "FR"),
            Features {
                buy: true,
                sell: true,
                swap: true,
                perpetuals: true,
                rewards: true,
                staking: true
            }
        );
        let policies = vec![
            FeaturePolicy::mock(),
            FeaturePolicy {
                feature: Feature::Sell,
                country_code: "DE".to_string(),
                ..FeaturePolicy::mock()
            },
            FeaturePolicy {
                feature: Feature::Rewards,
                ..FeaturePolicy::mock()
            },
            FeaturePolicy {
                feature: Feature::Staking,
                ..FeaturePolicy::mock()
            },
        ];
        assert_eq!(
            Features::for_country(&policies, "FR"),
            Features {
                buy: false,
                sell: true,
                swap: true,
                perpetuals: true,
                rewards: false,
                staking: false
            }
        );
        assert_eq!(
            Features::for_country(&policies, "DE"),
            Features {
                buy: true,
                sell: false,
                swap: true,
                perpetuals: true,
                rewards: true,
                staking: true
            }
        );
        assert_eq!(
            Features::for_country(&policies, "US"),
            Features {
                buy: true,
                sell: true,
                swap: true,
                perpetuals: true,
                rewards: true,
                staking: true
            }
        );
        let policies = vec![FeaturePolicy { is_enabled: true, ..FeaturePolicy::mock() }];
        assert_eq!(
            Features::for_country(&policies, "FR"),
            Features {
                buy: true,
                sell: true,
                swap: true,
                perpetuals: true,
                rewards: true,
                staking: true
            }
        );
    }
}
