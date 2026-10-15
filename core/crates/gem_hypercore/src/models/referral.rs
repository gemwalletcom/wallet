use chrono::{DateTime, Utc};
use gem_evm::address_deserializer::deserialize_ethereum_address_checksum;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_f64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Referral {
    pub referred_by: Option<ReferredBy>,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub cum_vlm: f64,
    pub referrer_state: Option<ReferrerState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferredBy {
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferrerState {
    pub data: Option<ReferrerData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferrerData {
    pub referral_states: Option<Vec<ReferralAddress>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferralAddress {
    #[serde(deserialize_with = "deserialize_ethereum_address_checksum")]
    pub user: String,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub time_joined: DateTime<Utc>,
    #[serde(flatten)]
    pub usdc: ReferralTokenState,
    pub token_to_state: Option<Vec<(i32, ReferralTokenState)>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferralTokenState {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub cum_vlm: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub cum_fees_rewarded_to_referrer: f64,
}

#[cfg(test)]
mod tests {
    use super::Referral;

    #[test]
    fn referral_need_to_trade_payload() {
        let referral: Referral = serde_json::from_str(include_str!("../../testdata/referral_need_to_trade.json")).unwrap();

        assert_eq!(referral.cum_vlm, 0.0);
        assert!(referral.referrer_state.unwrap().data.unwrap().referral_states.is_none());
    }

    #[test]
    fn referral_need_to_create_code_payload() {
        let referral: Referral = serde_json::from_str(include_str!("../../testdata/referral_need_to_create_code.json")).unwrap();

        assert!(referral.referrer_state.unwrap().data.is_none());
    }
}
