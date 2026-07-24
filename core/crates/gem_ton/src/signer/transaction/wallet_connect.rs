use std::collections::HashMap;
use std::str::FromStr;

use num_bigint::BigUint;
use primitives::{Chain, SignerError, SignerInput, TransferDataOutputType, WalletConnectCAIP2, unix_timestamp};
use serde::Deserialize;

use super::{
    message::DEFAULT_SEND_MODE,
    request::{TransferPayload, TransferRequest},
};
use crate::{
    address::Address,
    signer::TonSigner,
    tvm::{BagOfCells, CellArc},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WalletConnectRequest {
    #[serde(rename = "valid_until")]
    valid_until: Option<u64>,
    network: Option<String>,
    from: Option<String>,
    messages: Vec<WalletConnectMessage>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WalletConnectMessage {
    address: String,
    amount: String,
    payload: Option<String>,
    state_init: Option<String>,
    extra_currency: Option<HashMap<String, String>>,
}

impl TonSigner {
    pub(crate) fn sign_wallet_connect(&self, input: &SignerInput) -> Result<String, SignerError> {
        let extra = input.get_data_extra().map_err(SignerError::invalid_input)?;
        if extra.output_type != TransferDataOutputType::EncodedTransaction {
            return SignerError::invalid_input_err("TON WalletConnect requires encoded transaction output");
        }
        let data = extra
            .data
            .as_ref()
            .ok_or_else(|| SignerError::invalid_input("missing TON WalletConnect transaction data"))?;
        let request: WalletConnectRequest = serde_json::from_slice(data).map_err(|error| SignerError::invalid_input(format!("invalid TON WalletConnect request: {error}")))?;

        if Address::parse(&input.sender_address)? != *self.address() {
            return SignerError::invalid_input_err("TON sender does not match signing key");
        }
        Address::ensure_matches(request.from.as_deref(), &input.sender_address)?;

        if let Some(network) = request.network.as_deref()
            && WalletConnectCAIP2::get_reference(Chain::Ton).as_deref() != Some(network)
        {
            return SignerError::invalid_input_err("TON WalletConnect network does not match wallet network");
        }

        let expire_at = request.valid_until.map(parse_expire_at).transpose()?;
        let [message]: [WalletConnectMessage; 1] = request
            .messages
            .try_into()
            .map_err(|_| SignerError::invalid_input("TON WalletConnect requires exactly one message"))?;
        let sequence = input
            .metadata
            .get_sequence()
            .map_err(|error| SignerError::invalid_input(format!("invalid TON transaction metadata: {error}")))?;
        self.sign_requests(vec![message.into_request()?], sequence, expire_at)
    }
}

impl WalletConnectMessage {
    fn into_request(self) -> Result<TransferRequest, SignerError> {
        if self.extra_currency.as_ref().is_some_and(|currencies| !currencies.is_empty()) {
            return SignerError::invalid_input_err("TON extra currencies are not supported");
        }
        let (destination, bounceable) =
            Address::parse_user_friendly(&self.address).ok_or_else(|| SignerError::invalid_input("TON WalletConnect destination must be user-friendly"))?;
        Ok(TransferRequest {
            destination,
            value: BigUint::from_str(&self.amount)?,
            mode: DEFAULT_SEND_MODE,
            bounceable,
            comment: None,
            payload: parse_cell(self.payload, "payload")?.map(TransferPayload::Custom),
            state_init: parse_cell(self.state_init, "state init")?,
        })
    }
}

fn parse_cell(value: Option<String>, name: &str) -> Result<Option<CellArc>, SignerError> {
    value
        .map(|value| BagOfCells::parse_base64_root(&value).map_err(|error| SignerError::invalid_input(format!("invalid TON WalletConnect {name}: {error}"))))
        .transpose()
}

fn parse_expire_at(value: u64) -> Result<u32, SignerError> {
    let expire_at = u32::try_from(value).map_err(|_| SignerError::invalid_input("TON WalletConnect expiration does not fit in u32"))?;
    if value <= unix_timestamp() {
        return SignerError::invalid_input_err("TON WalletConnect transaction expired");
    }
    Ok(expire_at)
}

#[cfg(test)]
mod tests {
    use primitives::{Chain, ChainSigner, TransactionLoadInput, TransactionLoadMetadata};
    use serde_json::Value;

    use super::*;
    use crate::signer::{
        TonChainSigner,
        testkit::{TEST_PRIVATE_KEY, mock_signer},
    };

    fn input(data: &str) -> SignerInput {
        let signer = mock_signer();
        let mut input = TransactionLoadInput::mock_sign_data(Chain::Ton, data, TransferDataOutputType::EncodedTransaction);
        input.sender_address = signer.address().encode_non_bounceable();
        input.metadata = TransactionLoadMetadata::mock_ton(1);
        let fee = input.default_fee();
        SignerInput::new(input, fee)
    }

    fn sign(data: &str) -> Result<String, SignerError> {
        TonChainSigner.sign_data(&input(data), &hex::decode(TEST_PRIVATE_KEY).unwrap())
    }

    #[test]
    fn test_sign_wallet_connect() {
        let signer = mock_signer();
        let from = format!("0:{}", hex::encode(signer.address().hash_part()));
        let request = include_str!("../../../testdata/wallet_connect_dedust_send_message.json");
        let signed = sign(request).unwrap();
        assert!(BagOfCells::parse_base64_root(&signed).is_ok());

        let wrong_from = request.replace(&from, "0:33a14a5a9406979d59b9328898591660b8b1736342b11632efdcc911ab9057cf");
        assert_eq!(sign(&wrong_from).unwrap_err().to_string(), "Invalid input: TON from does not match signer address");

        let raw_destination = request.replace("EQDa4VOnTYlLvDJ0gZjNYm5PXfSmmtL6Vs6A_CZEtXCNICq_", &from);
        assert_eq!(
            sign(&raw_destination).unwrap_err().to_string(),
            "Invalid input: TON WalletConnect destination must be user-friendly"
        );

        let mut multiple: Value = serde_json::from_str(request).unwrap();
        let message = multiple["messages"][0].clone();
        multiple["messages"].as_array_mut().unwrap().push(message);
        assert_eq!(
            sign(&multiple.to_string()).unwrap_err().to_string(),
            "Invalid input: TON WalletConnect requires exactly one message"
        );
    }
}
