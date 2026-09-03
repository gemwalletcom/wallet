use super::amount;
use super::error::{PaymentDecoderError, Result};
use super::query;
use crate::{
    AssetId, Chain,
    payment::{Payment, PaymentAmount, PaymentRequest},
};

const TRANSFER_PATH: &str = "transfer";

const QUERY_AMOUNT: &str = "amount";
const QUERY_TEXT: &str = "text";
const QUERY_BODY: &str = "bin";
const QUERY_STATE_INIT: &str = "init";

pub fn decode(path: &str) -> Result<Payment> {
    let (path, query) = path.split_once('?').unwrap_or((path, ""));
    let parameters = query::parameters(query);

    if query::contains(&parameters, QUERY_BODY) || query::contains(&parameters, QUERY_STATE_INIT) {
        return Err(PaymentDecoderError::InvalidFormat("Unsupported transfer payload".to_string()));
    }

    Ok(Payment::Request(PaymentRequest {
        address: address(path)?,
        amount: query::value(&parameters, QUERY_AMOUNT)
            .and_then(|value| amount::exact_from_atomic(&value, Chain::Ton))
            .map(PaymentAmount::ExactValue),
        memo: query::value(&parameters, QUERY_TEXT),
        label: None,
        references: None,
        asset_id: Some(AssetId::from_chain(Chain::Ton)),
    }))
}

fn address(path: &str) -> Result<String> {
    let path = path.trim_matches('/');

    match path.split_once('/') {
        None if path.is_empty() || path == TRANSFER_PATH => Err(PaymentDecoderError::MissingField("address".to_string())),
        None => Ok(path.to_string()),
        Some((TRANSFER_PATH, address)) if !address.is_empty() && !address.contains('/') => Ok(address.to_string()),
        Some(_) => Err(PaymentDecoderError::InvalidFormat(format!("Not a transfer path: {path}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADDRESS: &str = "UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA";

    #[test]
    fn test_decode() {
        let ton = Payment::Request(PaymentRequest {
            address: ADDRESS.to_string(),
            asset_id: Some(AssetId::from_chain(Chain::Ton)),
            ..PaymentRequest::mock()
        });

        assert_eq!(
            decode(&format!("//transfer/{ADDRESS}?amount=1000000000&text=order+7")).unwrap(),
            Payment::Request(PaymentRequest {
                address: ADDRESS.to_string(),
                amount: Some(PaymentAmount::ExactValue("1".to_string())),
                memo: Some("order 7".to_string()),
                label: None,
                references: None,
                asset_id: Some(AssetId::from_chain(Chain::Ton)),
            })
        );
        assert_eq!(decode(&format!("//transfer/{ADDRESS}")).unwrap(), ton);
        assert_eq!(decode(ADDRESS).unwrap(), ton);
    }

    #[test]
    fn test_decode_refuses_what_it_cannot_sign() {
        assert_eq!(
            decode(&format!("//transfer/{ADDRESS}?amount=1&bin=te6cc")),
            Err(PaymentDecoderError::InvalidFormat("Unsupported transfer payload".to_string()))
        );
        assert_eq!(
            decode(&format!("//transfer/{ADDRESS}?amount=1&init=te6cc")),
            Err(PaymentDecoderError::InvalidFormat("Unsupported transfer payload".to_string()))
        );
        assert_eq!(
            decode(&format!("//transfer/{ADDRESS}?amount=1&BIN=te6cc")),
            Err(PaymentDecoderError::InvalidFormat("Unsupported transfer payload".to_string()))
        );
        assert!(decode("//invalid/format").is_err());
    }
}
