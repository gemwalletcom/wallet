use gem_tron::models::{TriggerSmartContractData, TronContractType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TronStepData {
    pub parameter: TriggerSmartContractData,
    #[serde(rename = "type")]
    pub transaction_type: TronContractType,
}

impl TronStepData {
    pub fn trigger_smart_contract(&self) -> Option<&TriggerSmartContractData> {
        (self.transaction_type == TronContractType::TriggerSmart).then_some(&self.parameter)
    }
}

#[cfg(test)]
mod tests {
    use crate::relay::model::RelayQuoteResponse;

    #[test]
    fn test_tron_step() {
        let response: RelayQuoteResponse = serde_json::from_str(include_str!("../testdata/quote_tron_usdt_to_base_usdc.json")).unwrap();
        let contract = response.get_tron_step().unwrap().trigger_smart_contract().unwrap();

        assert_eq!(contract.contract_address, "41f0623e1012177482912fb057e44e1a9769b1f588");
        assert_eq!(contract.call_value, None);
        assert_eq!(response.details.currency_out.amount, "976767");
        assert!(response.get_evm_step().is_none());

        let response: RelayQuoteResponse = serde_json::from_str(include_str!("../testdata/quote_tron_to_base_usdc.json")).unwrap();
        let contract = response.get_tron_step().unwrap().trigger_smart_contract().unwrap();

        assert_eq!(contract.call_value, Some(10_000_000));
    }
}
