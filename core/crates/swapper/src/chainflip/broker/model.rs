use gem_jsonrpc::types::JsonRpcResult;
use gem_polkadot::PolkadotAddress;
use num_bigint::BigUint;
use primitives::{Address, hex::decode_hex};
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_hex_str, serialize_biguint_to_hex_str};

use crate::SwapperError;

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

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum VaultSwapExtras {
    Evm(VaultSwapChainExtras),
    Tron(VaultSwapChainExtras),
    Solana(VaultSwapSolanaExtras),
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
    pub seed: String,
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
    pub data: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DepositChannelRequest {
    pub source_asset: ChainflipAsset,
    pub destination_asset: ChainflipAsset,
    pub destination_address: String,
    pub boost_fee: Option<u32>,
    pub refund_parameters: RefundParameters,
    pub dca_parameters: Option<DcaParameters>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DepositChannelResponse {
    pub address: String,
}

impl DepositChannelResponse {
    pub fn from_jsonrpc(data: &[u8]) -> Option<Self> {
        serde_json::from_slice::<JsonRpcResult<Self>>(data).ok()?.take().ok()
    }

    pub fn deposit_address(&self) -> Result<String, SwapperError> {
        let address = match PolkadotAddress::try_parse(&self.address) {
            Some(address) => address,
            None => {
                let account_id: [u8; 32] = decode_hex(&self.address).ok().and_then(|bytes| bytes.try_into().ok()).ok_or(SwapperError::InvalidRoute)?;
                PolkadotAddress::from_public_key(account_id)
            }
        };
        Ok(address.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::testkit::signer_mock::TEST_EVM_RECIPIENT;

    #[test]
    fn test_deposit_channel_address() {
        let response: DepositChannelResponse = serde_json::from_str(include_str!("./test/deposit_channel_assethub.json")).unwrap();
        let address = "12NfZE9HvsZseFTYGAcwdTyw5NDxVNZUVsyggP5XTQN8j9xB";

        assert_eq!(response.deposit_address().unwrap(), address);
        assert_eq!(DepositChannelResponse { address: address.to_string() }.deposit_address().unwrap(), address);
        assert_eq!(DepositChannelResponse { address: "0x1234".to_string() }.deposit_address(), Err(SwapperError::InvalidRoute));
        assert_eq!(DepositChannelResponse { address: TEST_EVM_RECIPIENT.to_string() }.deposit_address(), Err(SwapperError::InvalidRoute));
    }

    #[test]
    fn test_deposit_channel_from_jsonrpc() {
        let channel = br#"{"jsonrpc":"2.0","id":1,"result":{"address":"0x3cc22d40b17571ee448414b0a21c7e8f6f2b96aaab95625363918412b4721550","issued_block":15242255,"channel_id":6}}"#;
        let error = br#"{"jsonrpc":"2.0","id":1,"error":{"code":-32603,"message":"Invalid destination address"}}"#;

        assert_eq!(DepositChannelResponse::from_jsonrpc(channel).unwrap().deposit_address().unwrap(), "12NfZE9HvsZseFTYGAcwdTyw5NDxVNZUVsyggP5XTQN8j9xB");
        assert!(DepositChannelResponse::from_jsonrpc(error).is_none());
    }

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
