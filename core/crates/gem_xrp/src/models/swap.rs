use num_bigint::BigUint;
use primitives::{AssetId, TransactionSwapReferralFee};
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_str, serialize_biguint};

const BATCH_FEE_MULTIPLIER: u64 = 2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XrpSwap {
    #[serde(serialize_with = "serialize_biguint", deserialize_with = "deserialize_biguint_from_str")]
    pub send_max: BigUint,
    #[serde(serialize_with = "serialize_biguint", deserialize_with = "deserialize_biguint_from_str")]
    pub amount: BigUint,
    #[serde(serialize_with = "serialize_biguint", deserialize_with = "deserialize_biguint_from_str")]
    pub deliver_min: BigUint,
    pub paths: Vec<Vec<XrpPathStep>>,
    pub fee: Option<TransactionSwapReferralFee>,
    pub trust_line: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XrpPathStep {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
}

impl XrpSwap {
    pub fn input_value(&self, from_asset: &AssetId) -> BigUint {
        let input_fee = self.fee.as_ref().filter(|fee| fee.asset_id == *from_asset).map_or(BigUint::ZERO, |fee| fee.value.clone());
        &self.send_max + input_fee
    }

    pub fn fee_multiplier(&self) -> u64 {
        match 1 + u64::from(self.trust_line) + u64::from(self.fee.is_some()) {
            1 => 1,
            transactions => BATCH_FEE_MULTIPLIER + transactions,
        }
    }
}
