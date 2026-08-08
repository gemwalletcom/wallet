use core::str;

use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_str, deserialize_option_biguint_from_str, deserialize_u64_from_str};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionBroadcast {
    pub hash: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockHeader {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub number: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Block {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub number: u64,
    pub extrinsics: Vec<Extrinsic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extrinsic {
    pub hash: String,
    pub method: ExtrinsicMethod,
    pub info: ExtrinsicInfo,
    pub success: bool,
    pub args: ExtrinsicArguments,
    pub signature: Option<Signature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtrinsicMethod {
    pub pallet: String,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtrinsicInfo {
    #[serde(default, deserialize_with = "deserialize_option_biguint_from_str")]
    pub partial_fee: Option<BigUint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExtrinsicArguments {
    Transfer(ExtrinsicTransfer),
    Transfers(ExtrinsicCalls),
    Timestamp(ExtrinsicTimestamp),
    Other(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtrinsicTimestamp {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub now: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtrinsicTransfer {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub value: BigUint,
    pub dest: AddressId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtrinsicCalls {
    pub calls: Vec<Call>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Call {
    pub method: ExtrinsicMethod,
    pub args: CallArgs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallArgs {
    pub dest: AddressId,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub value: BigUint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressId {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub signer: AddressId,
}
