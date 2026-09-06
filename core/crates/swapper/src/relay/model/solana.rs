use primitives::SolanaInstruction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaStepData {
    pub instructions: Vec<SolanaInstruction>,
    #[serde(default)]
    pub address_lookup_table_addresses: Vec<String>,
}

#[cfg(test)]
mod tests {
    use crate::relay::model::RelayQuoteResponse;

    #[test]
    fn test_solana_step() {
        let response: RelayQuoteResponse = serde_json::from_str(include_str!("../testdata/quote_sol_to_base_usdc.json")).unwrap();
        let solana = response.get_solana_step().unwrap();

        assert_eq!(solana.instructions.len(), 1);
        assert_eq!(solana.instructions[0].accounts.len(), 5);
        assert_eq!(solana.address_lookup_table_addresses, vec!["Hm9fUgcn7qwDaiNTFiGh6pNtVATgnaRcmK6Bbx6EMZfP".to_string()]);
        assert!(response.get_evm_step().is_none());
    }
}
