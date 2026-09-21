use crate::{FEE_PAYER_SIGNATURE_TYPE, FUNGIBLE_ASSET_DEPOSIT_EVENT, FUNGIBLE_ASSET_WITHDRAW_EVENT, NO_ACCOUNT_SIGNATURE_TYPE, SIMULATION_FEE_PAYER_ADDRESS, STAKE_DEPOSIT_EVENT, STAKE_WITHDRAW_EVENT};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_str, deserialize_option_biguint_from_str, deserialize_option_u64_from_str, deserialize_u64_from_str};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: Option<String>,
    pub sender: Option<String>,
    pub success: bool,
    pub vm_status: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_u64_from_str")]
    pub gas_used: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_option_u64_from_str")]
    pub gas_unit_price: Option<u64>,
    pub events: Option<Vec<Event>>,
    pub payload: Option<TransactionPayload>,
    #[serde(rename = "type", default)]
    pub transaction_type: Option<String>,
    pub sequence_number: Option<String>,
    #[serde(default, deserialize_with = "deserialize_u64_from_str")]
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub guid: Guid,
    pub data: Option<serde_json::Value>,
    #[serde(rename = "type")]
    pub event_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountData {
    #[serde(default, deserialize_with = "deserialize_option_biguint_from_str")]
    pub amount: Option<BigUint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationPoolAddStakeData {
    pub pool_address: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub amount_added: BigUint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationPoolUnlockStakeData {
    pub pool_address: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub amount_unlocked: BigUint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guid {
    pub account_address: String,
}

impl Event {
    pub fn get_amount(&self) -> Option<BigUint> {
        let data = self.data.clone()?;
        match self.event_type.as_str() {
            STAKE_WITHDRAW_EVENT | STAKE_DEPOSIT_EVENT | FUNGIBLE_ASSET_WITHDRAW_EVENT | FUNGIBLE_ASSET_DEPOSIT_EVENT => serde_json::from_value::<AmountData>(data).ok()?.amount,
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPayload {
    pub function: Option<String>,
    #[serde(default)]
    pub type_arguments: Vec<String>,
    #[serde(default)]
    pub arguments: Vec<serde_json::Value>,
    #[serde(rename = "type")]
    pub payload_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSignature {
    #[serde(rename = "type")]
    pub signature_type: String,
    pub public_key: Option<String>,
    pub signature: Option<String>,
}

impl TransactionSignature {
    pub fn no_account() -> Self {
        TransactionSignature {
            signature_type: NO_ACCOUNT_SIGNATURE_TYPE.to_string(),
            public_key: None,
            signature: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SponsoredSimulationSignature {
    #[serde(rename = "type")]
    pub signature_type: String,
    pub sender: TransactionSignature,
    pub secondary_signer_addresses: Vec<String>,
    pub secondary_signers: Vec<TransactionSignature>,
    pub fee_payer_address: String,
    pub fee_payer_signer: TransactionSignature,
}

impl SponsoredSimulationSignature {
    pub fn unsigned() -> Self {
        Self {
            signature_type: FEE_PAYER_SIGNATURE_TYPE.to_string(),
            sender: TransactionSignature::no_account(),
            secondary_signer_addresses: vec![],
            secondary_signers: vec![],
            fee_payer_address: SIMULATION_FEE_PAYER_ADDRESS.to_string(),
            fee_payer_signer: TransactionSignature::no_account(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateTransactionQuery {
    pub estimate_max_gas_amount: bool,
    pub estimate_gas_unit_price: bool,
    pub estimate_prioritized_gas_unit_price: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransactionSimulation {
    pub expiration_timestamp_secs: String,
    pub gas_unit_price: String,
    pub max_gas_amount: String,
    pub payload: TransactionPayload,
    pub sender: String,
    pub sequence_number: String,
    pub signature: SponsoredSimulationSignature,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub hash: Option<String>,
    pub message: Option<String>,
    pub error_code: Option<String>,
    pub vm_error_code: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTransactionBcsRequest {
    pub bcs: String,
    #[serde(rename = "bcsEncoding")]
    pub bcs_encoding: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionBroadcast {
    pub hash: String,
}
