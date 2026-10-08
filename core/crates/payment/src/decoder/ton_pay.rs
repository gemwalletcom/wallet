use super::amount;
use super::error::{PaymentDecoderError, Result};
use super::query;
use primitives::{
    AssetId, Chain,
    payment::{Payment, PaymentAmount, PaymentRequest},
};

const TRANSFER_PATH: &str = "transfer";

const QUERY_AMOUNT: &str = "amount";
const QUERY_TEXT: &str = "text";
const QUERY_JETTON: &str = "jetton";
const QUERY_BODY: &str = "bin";
const QUERY_STATE_INIT: &str = "init";

pub fn decode(path: &str) -> Result<Payment> {
    let (path, query) = path.split_once('?').unwrap_or((path, ""));
    let parameters = query::parameters(query);

    if query::contains(&parameters, QUERY_BODY) || query::contains(&parameters, QUERY_STATE_INIT) {
        return Err(PaymentDecoderError::InvalidFormat("Unsupported transfer payload".to_string()));
    }

    let jetton = query::value(&parameters, QUERY_JETTON);
    let amount = query::value(&parameters, QUERY_AMOUNT).and_then(|value| match &jetton {
        Some(_) => amount::atomic(&value).map(|value| PaymentAmount::AtomicValue { value }),
        None => amount::exact_from_atomic(&value, Chain::Ton).map(|value| PaymentAmount::ExactValue { value }),
    });

    Ok(Payment::Request {
        request: PaymentRequest {
            address: address(path)?,
            amount,
            memo: query::value(&parameters, QUERY_TEXT),
            label: None,
            references: None,
            asset_id: Some(AssetId::from(Chain::Ton, jetton)),
        },
    })
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
    use primitives::asset_constants::{TON_DUST_ASSET_ID, TON_DUST_TOKEN_ID};

    const ADDRESS: &str = "UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA";

    #[test]
    fn test_decode() {
        let ton = Payment::Request {
            request: PaymentRequest {
                address: ADDRESS.to_string(),
                asset_id: Some(AssetId::from_chain(Chain::Ton)),
                ..PaymentRequest::mock()
            },
        };

        assert_eq!(
            decode(&format!("//transfer/{ADDRESS}?amount=1000000000&text=order+7")).unwrap(),
            Payment::Request {
                request: PaymentRequest {
                    address: ADDRESS.to_string(),
                    amount: Some(PaymentAmount::ExactValue { value: "1".to_string() }),
                    memo: Some("order 7".to_string()),
                    label: None,
                    references: None,
                    asset_id: Some(AssetId::from_chain(Chain::Ton)),
                }
            }
        );
        assert_eq!(decode(&format!("//transfer/{ADDRESS}")).unwrap(), ton);
        assert_eq!(decode(ADDRESS).unwrap(), ton);
    }

    #[test]
    fn test_decode_jetton() {
        assert_eq!(
            decode(&format!("//transfer/{ADDRESS}?jetton={TON_DUST_TOKEN_ID}&amount=5000000&text=hello")).unwrap(),
            Payment::Request {
                request: PaymentRequest {
                    address: ADDRESS.to_string(),
                    amount: Some(PaymentAmount::AtomicValue { value: 5_000_000u32.into() }),
                    memo: Some("hello".to_string()),
                    label: None,
                    references: None,
                    asset_id: Some(TON_DUST_ASSET_ID.clone()),
                }
            },
            "the amount is in the jetton's units, not nanoTON"
        );
    }

    #[test]
    fn test_decode_refuses_what_it_cannot_sign() {
        assert_eq!(decode(&format!("//transfer/{ADDRESS}?amount=1&bin=te6cc")), Err(PaymentDecoderError::InvalidFormat("Unsupported transfer payload".to_string())));
        assert_eq!(
            decode(&format!("//transfer/{ADDRESS}?amount=1&init=te6cc")),
            Err(PaymentDecoderError::InvalidFormat("Unsupported transfer payload".to_string()))
        );
        assert_eq!(decode(&format!("//transfer/{ADDRESS}?amount=1&BIN=te6cc")), Err(PaymentDecoderError::InvalidFormat("Unsupported transfer payload".to_string())));
        assert!(decode("//invalid/format").is_err());
    }
}
