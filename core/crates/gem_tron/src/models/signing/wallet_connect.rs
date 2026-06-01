use gem_hash::sha2::sha256;
use primitives::{SignerError, SignerInput, TransferDataOutputType};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_serializers::hex_bytes;

use super::RawDataJson;

pub(crate) struct WalletConnectPayload {
    transaction: WalletConnectTransaction,
    output_type: TransferDataOutputType,
}

impl WalletConnectPayload {
    pub(crate) fn parse(input: &SignerInput) -> Result<Self, SignerError> {
        let extra = input.get_data_extra().map_err(SignerError::invalid_input)?;
        let data = extra.data.as_ref().ok_or_else(|| SignerError::invalid_input("Missing transaction data"))?;
        let payload: WalletConnectRequest = serde_json::from_slice(data)?;

        Ok(Self {
            transaction: payload.transaction,
            output_type: extra.output_type.clone(),
        })
    }

    pub(crate) fn transaction_hash(&self) -> Result<[u8; 32], SignerError> {
        self.transaction.hash()
    }

    pub(crate) fn into_output(self, transaction_hash: [u8; 32], signature_hex: String) -> Result<String, SignerError> {
        match self.output_type {
            TransferDataOutputType::Signature => Ok(signature_hex),
            TransferDataOutputType::EncodedTransaction => self.transaction.into_signed_json(hex::encode(transaction_hash), signature_hex),
        }
    }
}

#[derive(Deserialize)]
struct WalletConnectRequest {
    transaction: WalletConnectTransaction,
}

#[derive(Deserialize, Serialize)]
struct WalletConnectTransaction {
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_data: Option<Value>,
    #[serde(with = "hex_bytes")]
    raw_data_hex: Vec<u8>,
    #[serde(rename = "txID", skip_serializing_if = "Option::is_none")]
    transaction_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<Vec<String>>,
    // Preserve non-wire dApp fields like `visible`; only raw_data_hex is signed.
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl WalletConnectTransaction {
    fn hash(&self) -> Result<[u8; 32], SignerError> {
        let raw_data = self.raw_data_hex.as_slice();
        let transaction_hash = sha256(raw_data);
        let transaction_id = hex::encode(transaction_hash);

        match &self.transaction_id {
            Some(provided_transaction_id) if !provided_transaction_id.eq_ignore_ascii_case(&transaction_id) => {
                return SignerError::invalid_input_err("transaction ID does not match hash of raw_data_hex");
            }
            None if self.raw_data.is_none() => SignerError::invalid_input_err("Missing raw_data or transaction ID"),
            _ => Ok(()),
        }?;

        if let Some(raw_data_json) = &self.raw_data {
            // The transaction ID validates signed bytes and keeps rendered raw_data and raw_data_hex in sync.
            let encoded = serde_json::from_value::<RawDataJson>(raw_data_json.clone())?.encode()?;
            if encoded != raw_data {
                return SignerError::invalid_input_err("raw_data does not match raw_data_hex");
            }
        }

        Ok(transaction_hash)
    }

    fn into_signed_json(mut self, transaction_id: String, signature_hex: String) -> Result<String, SignerError> {
        self.signature = Some(vec![signature_hex]);
        self.transaction_id = Some(transaction_id);
        serde_json::to_string(&self).map_err(Into::into)
    }
}
