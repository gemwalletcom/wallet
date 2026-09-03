use super::amount;
use super::error::{PaymentDecoderError, Result};
use super::query;
use crate::{
    AssetId, Chain,
    payment::{Payment, PaymentAmount, PaymentRequest},
};

const REQUIRED_PARAMETER_PREFIX: &str = "req-";
const QUERY_AMOUNT: &str = "amount";
const QUERY_MEMO: &str = "memo";
const QUERY_LABEL: &str = "label";

pub fn decode(chain: Option<Chain>, path: &str) -> Result<Payment> {
    Ok(Payment::Request(get_request(chain, path)?))
}

pub fn get_request(chain: Option<Chain>, path: &str) -> Result<PaymentRequest> {
    let (address, query) = path.split_once('?').unwrap_or((path, ""));
    let parameters = query::parameters(query);
    if let Some((required, _)) = parameters.iter().find(|(key, _)| key.starts_with(REQUIRED_PARAMETER_PREFIX)) {
        return Err(PaymentDecoderError::InvalidFormat(format!("Unsupported required parameter: {required}")));
    }

    if address.is_empty() {
        return Err(PaymentDecoderError::MissingField("address".to_string()));
    }

    Ok(PaymentRequest {
        address: address.to_string(),
        amount: query::value(&parameters, QUERY_AMOUNT)
            .and_then(|value| match chain {
                Some(chain) => amount::exact(&value, chain),
                None => amount::decimal(&value),
            })
            .map(PaymentAmount::ExactValue),
        memo: query::value(&parameters, QUERY_MEMO),
        label: query::value(&parameters, QUERY_LABEL),
        references: None,
        asset_id: chain.map(AssetId::from_chain),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const BITCOIN_ADDRESS: &str = "175tWpb8K1S7NmH4Zx6rewF9WQrcZv245W";

    #[test]
    fn test_decode() {
        let bitcoin = Payment::Request(PaymentRequest {
            address: BITCOIN_ADDRESS.to_string(),
            asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
            ..PaymentRequest::mock()
        });

        assert_eq!(decode(Some(Chain::Bitcoin), BITCOIN_ADDRESS).unwrap(), bitcoin);
        assert_eq!(decode(Some(Chain::Bitcoin), &format!("{BITCOIN_ADDRESS}?dontexist=")).unwrap(), bitcoin);
        assert_eq!(decode(Some(Chain::Bitcoin), &format!("{BITCOIN_ADDRESS}?amount=&memo=&label=")).unwrap(), bitcoin);

        assert_eq!(
            decode(Some(Chain::Bitcoin), &format!("{BITCOIN_ADDRESS}?amount=50&label=Luke-Jr&message=Donation%20for%20xyz")).unwrap(),
            Payment::Request(PaymentRequest {
                address: BITCOIN_ADDRESS.to_string(),
                amount: Some(PaymentAmount::ExactValue("50".to_string())),
                label: Some("Luke-Jr".to_string()),
                asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
                ..PaymentRequest::mock()
            })
        );
        assert_eq!(
            decode(Some(Chain::Bitcoin), &format!("{BITCOIN_ADDRESS}?memo=see%20http%3A%2F%2Fx.com")).unwrap(),
            Payment::Request(PaymentRequest {
                address: BITCOIN_ADDRESS.to_string(),
                memo: Some("see http://x.com".to_string()),
                asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
                ..PaymentRequest::mock()
            })
        );
        assert_eq!(
            decode(Some(Chain::Doge), "DH5yaieqoZN36fDVciNyRueRGvGLR3mr7L?amount=42").unwrap(),
            Payment::Request(PaymentRequest {
                address: "DH5yaieqoZN36fDVciNyRueRGvGLR3mr7L".to_string(),
                amount: Some(PaymentAmount::ExactValue("42".to_string())),
                asset_id: Some(AssetId::from_chain(Chain::Doge)),
                ..PaymentRequest::mock()
            })
        );
        assert_eq!(
            decode(None, &format!("{BITCOIN_ADDRESS}?amount=50.72")).unwrap(),
            Payment::Request(PaymentRequest {
                address: BITCOIN_ADDRESS.to_string(),
                amount: Some(PaymentAmount::ExactValue("50.72".to_string())),
                ..PaymentRequest::mock()
            })
        );
    }

    #[test]
    fn test_decode_refuses_what_it_cannot_sign() {
        assert_eq!(
            decode(Some(Chain::Bitcoin), &format!("{BITCOIN_ADDRESS}?req-somethingyoudontunderstand=50")),
            Err(PaymentDecoderError::InvalidFormat(
                "Unsupported required parameter: req-somethingyoudontunderstand".to_string()
            ))
        );
        assert_eq!(
            decode(Some(Chain::Bitcoin), &format!("{BITCOIN_ADDRESS}?req-dontexist=")),
            Err(PaymentDecoderError::InvalidFormat("Unsupported required parameter: req-dontexist".to_string()))
        );
        assert_eq!(
            decode(Some(Chain::Bitcoin), &format!("{BITCOIN_ADDRESS}?REQ-Pop=initiatingapp%3A")),
            Err(PaymentDecoderError::InvalidFormat("Unsupported required parameter: req-pop".to_string()))
        );
    }
}
