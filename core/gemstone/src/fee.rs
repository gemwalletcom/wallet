use num_bigint::BigInt;
use primitives::{AssetId, CustomFee, GasPriceType};

use crate::models::gateway::GemGasPriceType;
use crate::models::transaction::GemTransactionInputType;
use crate::services::confirm::rules as confirm_rules;

#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct GemCustomFee {
    pub fee_value: BigInt,
    pub max_rate: BigInt,
    pub is_over_max: bool,
}

#[derive(Default, uniffi::Object)]
pub struct GemFeeService {}

#[uniffi::export]
impl GemFeeService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn custom_gas_price(&self, base: GemGasPriceType, gas_price: BigInt) -> GemGasPriceType {
        custom_gas_price(base, gas_price)
    }

    pub fn custom_fee_estimate(&self, rate: Option<BigInt>, loaded_fee: BigInt, base_total: BigInt, normal_total: BigInt, max_multiplier: u32) -> GemCustomFee {
        let fee = CustomFee::calculate(rate, loaded_fee, base_total, normal_total, max_multiplier);

        GemCustomFee {
            fee_value: fee.fee_value,
            max_rate: fee.max_rate,
            is_over_max: fee.is_over_max,
        }
    }

    pub fn default_priority(&self, input_type: GemTransactionInputType) -> String {
        confirm_rules::default_fee_priority(input_type)
    }

    pub fn is_insufficient_network_fee(&self, fee_asset_id: AssetId, fee_available: String) -> bool {
        confirm_rules::is_insufficient_network_fee(fee_asset_id, &fee_available)
    }
}

pub(crate) fn custom_gas_price(base: GemGasPriceType, gas_price: BigInt) -> GemGasPriceType {
    let base: GasPriceType = base.into();
    base.custom(gas_price).into()
}
