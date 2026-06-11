use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_hex_str, serialize_biguint_to_hex_str};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct ChainflipAsset {
    pub chain: String,
    pub asset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RefundParameters {
    pub retry_duration: u32,
    pub refund_address: String,
    pub min_price: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DcaParameters {
    pub number_of_chunks: u32,
    pub chunk_interval: u32,
}

#[derive(Debug)]
pub enum VaultSwapExtras {
    Evm(VaultSwapEvmExtras),
    Bitcoin(VaultSwapBtcExtras),
    Solana(VaultSwapSolanaExtras),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSwapEvmExtras {
    pub chain: String,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str", serialize_with = "serialize_biguint_to_hex_str")]
    pub input_amount: BigUint,
    pub refund_parameters: RefundParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSwapBtcExtras {
    pub chain: String,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str", serialize_with = "serialize_biguint_to_hex_str")]
    pub min_output_amount: BigUint,
    pub retry_duration: u32,
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
    Evm(EvmVaultSwapResponse),
    Bitcoin(BitcoinVaultSwapResponse),
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
pub struct BitcoinVaultSwapResponse {
    pub nulldata_payload: String,
    pub deposit_address: String,
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
