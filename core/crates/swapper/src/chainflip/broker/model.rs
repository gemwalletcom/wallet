use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_hex_str, serialize_biguint_to_hex_str};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct ChainflipAsset {
    pub chain: String,
    pub asset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RefundParameters {
    pub retry_duration: u32,
    pub refund_address: String,
    pub min_price: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_oracle_price_slippage: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DcaParameters {
    pub number_of_chunks: u32,
    pub chunk_interval: u32,
}

#[derive(Debug)]
pub enum VaultSwapExtras {
    Evm(VaultSwapChainExtras),
    Tron(VaultSwapChainExtras),
    Solana(VaultSwapSolanaExtras),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSwapChainExtras {
    pub chain: String,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str", serialize_with = "serialize_biguint_to_hex_str")]
    pub input_amount: BigUint,
    pub refund_parameters: RefundParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSwapSolanaExtras {
    pub chain: String,
    pub from: String,
    pub seed: String, // random bytes (up to 32 bytes) in hex string
    pub input_amount: u64,
    pub refund_parameters: RefundParameters,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum VaultSwapResponse {
    Tron(TronVaultSwapResponse),
    Evm(EvmVaultSwapResponse),
    Solana(SolanaVaultSwapResponse),
}

#[derive(Debug, Clone, Deserialize)]
pub struct EvmVaultSwapResponse {
    pub calldata: String,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str", serialize_with = "serialize_biguint_to_hex_str")]
    pub value: BigUint,
    pub to: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SolanaVaultSwapResponse {
    pub program_id: String,
    pub accounts: Vec<AccountMeta>,
    pub data: String, // hex string
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountMeta {
    pub is_signer: bool,
    pub is_writable: bool,
    pub pubkey: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TronVaultSwapResponse {
    pub calldata: String,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
    pub value: BigUint,
    pub to: String,
    pub note: String,
    pub source_token_address: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tron_vault_swap_response_deserializes_before_evm() {
        let response: VaultSwapResponse = serde_json::from_value(serde_json::json!({
            "calldata": "0xa9059cbb",
            "value": "0x0",
            "to": "0x2523ae929fecd9d665f472f59b99a8ce6b179510",
            "note": "0x0300",
            "source_token_address": "0xeca9bc828a3005b9a3b909f2cc5c2a54794de05f"
        }))
        .unwrap();

        let VaultSwapResponse::Tron(response) = response else {
            panic!("expected Tron vault swap response");
        };
        assert_eq!(response.value, BigUint::from(0u32));
        assert_eq!(response.note, "0x0300");
        assert_eq!(response.source_token_address, Some("0xeca9bc828a3005b9a3b909f2cc5c2a54794de05f".to_string()));
    }

    #[test]
    fn test_evm_vault_swap_response_keeps_original_shape() {
        let response: VaultSwapResponse = serde_json::from_value(serde_json::json!({
            "calldata": "0x1234",
            "value": "0x3e8",
            "to": "0x1111111111111111111111111111111111111111"
        }))
        .unwrap();

        let VaultSwapResponse::Evm(response) = response else {
            panic!("expected EVM vault swap response");
        };
        assert_eq!(response.value, BigUint::from(1000u32));
        assert_eq!(response.to, "0x1111111111111111111111111111111111111111");
    }
}
