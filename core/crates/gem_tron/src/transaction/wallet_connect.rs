use gem_hash::sha2::sha256;
use primitives::{ApprovalData, SignerError};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_serializers::hex_bytes;

use super::RawDataJson;

#[derive(Deserialize)]
pub(crate) struct WalletConnectRequest {
    pub(crate) transaction: WalletConnectTransaction,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct WalletConnectTransaction {
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_data: Option<Value>,
    #[serde(with = "hex_bytes")]
    raw_data_hex: Vec<u8>,
    #[serde(rename = "txID", skip_serializing_if = "Option::is_none")]
    pub(crate) transaction_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) signature: Option<Vec<String>>,
    // Preserve non-wire dApp fields like `visible`; only raw_data_hex is signed.
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl WalletConnectTransaction {
    pub(crate) fn validate(&self) -> Result<([u8; 32], RawDataJson), SignerError> {
        let raw_data = self.raw_data_hex.as_slice();
        let transaction_hash = sha256(raw_data);
        let transaction_id = hex::encode(transaction_hash);

        let raw_data_json = self.raw_data.as_ref().ok_or_else(|| SignerError::invalid_input("Missing raw_data"))?;

        if let Some(provided_transaction_id) = &self.transaction_id
            && !provided_transaction_id.eq_ignore_ascii_case(&transaction_id)
        {
            return SignerError::invalid_input_err("transaction ID does not match hash of raw_data_hex");
        }

        let decoded = serde_json::from_value::<RawDataJson>(raw_data_json.clone())?;
        if decoded.encode()? != raw_data {
            return SignerError::invalid_input_err("raw_data does not match raw_data_hex");
        }

        Ok((transaction_hash, decoded))
    }
}

pub fn decode_wallet_connect_approval(data: &str) -> Result<Option<ApprovalData>, SignerError> {
    let payload: WalletConnectRequest = serde_json::from_str(data)?;
    let (_, raw_data) = payload.transaction.validate()?;
    raw_data.approval()
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;

    const PAYLOAD: &str = include_str!("../../../gem_wallet_connect/testdata/tron_send_transaction.json");

    fn payload_with_calldata(data: &str) -> String {
        let mut payload: Value = serde_json::from_str(PAYLOAD).unwrap();
        payload["transaction"]["raw_data"]["contract"][0]["parameter"]["value"]["data"] = Value::String(data.to_string());
        encode_payload(payload)
    }

    fn encode_payload(mut payload: Value) -> String {
        let raw_data = serde_json::from_value::<RawDataJson>(payload["transaction"]["raw_data"].clone()).unwrap().encode().unwrap();
        payload["transaction"]["raw_data_hex"] = Value::String(hex::encode(&raw_data));
        payload["transaction"]["txID"] = Value::String(hex::encode(sha256(&raw_data)));
        payload.to_string()
    }

    #[test]
    fn test_decode_wallet_connect_approval() {
        let selector_and_spender = "095ea7b300000000000000000000000060e00625a95cbc180f290e2611c826f90eeba56f";
        for (amount, is_unlimited) in [(BigUint::from(0u32), false), (BigUint::from(100u32), false), (BigUint::from_bytes_be(&[0xff; 32]), true)] {
            let payload = payload_with_calldata(&format!("{selector_and_spender}{amount:064x}"));
            assert_eq!(
                decode_wallet_connect_approval(&payload).unwrap(),
                Some(ApprovalData {
                    token: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string(),
                    spender: "TJoSEwEqt7cT3TUwmEoUYnYs5cZR3xSukM".to_string(),
                    value: amount,
                    is_unlimited,
                })
            );
        }
        assert!(decode_wallet_connect_approval(PAYLOAD).unwrap().is_some());
        assert_eq!(decode_wallet_connect_approval(&payload_with_calldata("deadbeef")).unwrap(), None);
        assert!(decode_wallet_connect_approval(&payload_with_calldata("095ea7b3abcd")).is_err());
        assert!(decode_wallet_connect_approval(&payload_with_calldata(&format!("095ea7b3{}{}", "ff".repeat(32), "00".repeat(32)))).is_err());
    }

    #[test]
    fn test_decode_wallet_connect_approval_with_additional_effects() {
        for field in ["call_value", "call_token_value"] {
            let mut payload: Value = serde_json::from_str(PAYLOAD).unwrap();
            payload["transaction"]["raw_data"]["contract"][0]["parameter"]["value"][field] = Value::from(1);
            assert!(decode_wallet_connect_approval(&encode_payload(payload)).is_err());
        }
        let mut payload: Value = serde_json::from_str(PAYLOAD).unwrap();
        let contract = payload["transaction"]["raw_data"]["contract"][0].clone();
        payload["transaction"]["raw_data"]["contract"].as_array_mut().unwrap().push(contract);
        assert_eq!(decode_wallet_connect_approval(&encode_payload(payload)).unwrap(), None);
    }

    #[test]
    fn test_decode_wallet_connect_approval_rejects_mismatched_signed_data() {
        let payload = PAYLOAD.replace("095ea7b3", "a9059cbb").replacen("a9059cbb", "095ea7b3", 1);
        assert!(decode_wallet_connect_approval(&payload).is_err());
        let mut payload: Value = serde_json::from_str(PAYLOAD).unwrap();
        payload["transaction"].as_object_mut().unwrap().remove("txID");
        payload["transaction"]["raw_data"]["contract"][0]["parameter"]["value"]["data"] = Value::String("deadbeef".to_string());
        assert!(decode_wallet_connect_approval(&payload.to_string()).is_err());
    }
}
