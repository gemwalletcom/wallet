use super::amount;
use super::error::{PaymentDecoderError, Result};
use super::query;
use crate::{
    AssetId, Chain,
    payment::{Payment, PaymentAmount, PaymentRequest},
};

const REQUIRED_PARAMETER_PREFIX: &str = "req-";
const AUTHORITY_PREFIX: &str = "//";
const QUERY_AMOUNT: &str = "amount";
const QUERY_MEMO: &str = "memo";
const QUERY_DESTINATION_TAG: &str = "dt";

pub fn decode(chain: Option<Chain>, path: &str) -> Result<Payment> {
    let asset_id = chain.map(AssetId::from_chain);
    let path = path.strip_prefix(AUTHORITY_PREFIX).unwrap_or(path);

    let Some((address, query)) = path.split_once('?') else {
        return Ok(Payment::Request(PaymentRequest {
            address: path.to_string(),
            amount: None,
            memo: None,
            references: None,
            asset_id,
        }));
    };

    let parameters = query::parameters(query);
    if let Some(required) = parameters.keys().find(|key| key.starts_with(REQUIRED_PARAMETER_PREFIX)) {
        return Err(PaymentDecoderError::InvalidFormat(format!("Unsupported required parameter: {required}")));
    }

    let is_xrp = asset_id.as_ref().is_some_and(|asset_id| asset_id.chain == Chain::Xrp);
    let memo = query::value(&parameters, QUERY_MEMO).or_else(|| query::value(&parameters, QUERY_DESTINATION_TAG).filter(|_| is_xrp));

    let amount = query::value(&parameters, QUERY_AMOUNT)
        .and_then(|value| match asset_id.as_ref() {
            Some(asset_id) => amount::exact(&value, asset_id.chain),
            None => amount::decimal(&value),
        })
        .map(PaymentAmount::ExactValue);

    Ok(Payment::Request(PaymentRequest {
        address: address.to_string(),
        amount,
        memo,
        references: None,
        asset_id,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BITCOIN_ADDRESS: &str = "175tWpb8K1S7NmH4Zx6rewF9WQrcZv245W";
    const XRP_ADDRESS: &str = "rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh";

    #[test]
    fn test_decode() {
        let bitcoin = Payment::Request(PaymentRequest {
            address: BITCOIN_ADDRESS.to_string(),
            asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
            ..PaymentRequest::mock()
        });

        assert_eq!(decode(Some(Chain::Bitcoin), BITCOIN_ADDRESS).unwrap(), bitcoin);
        assert_eq!(decode(Some(Chain::Bitcoin), &format!("//{BITCOIN_ADDRESS}")).unwrap(), bitcoin);
        assert_eq!(decode(Some(Chain::Bitcoin), &format!("{BITCOIN_ADDRESS}?label=Luke-Jr")).unwrap(), bitcoin);
        assert_eq!(decode(Some(Chain::Bitcoin), &format!("{BITCOIN_ADDRESS}?dontexist=")).unwrap(), bitcoin);
        assert_eq!(decode(Some(Chain::Bitcoin), &format!("{BITCOIN_ADDRESS}?amount=&memo=")).unwrap(), bitcoin);

        assert_eq!(
            decode(Some(Chain::Bitcoin), &format!("{BITCOIN_ADDRESS}?amount=50&label=Luke-Jr&message=Donation%20for%20xyz")).unwrap(),
            Payment::Request(PaymentRequest {
                address: BITCOIN_ADDRESS.to_string(),
                amount: Some(PaymentAmount::ExactValue("50".to_string())),
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
    fn test_decode_destination_tag() {
        let payment = Payment::Request(PaymentRequest {
            address: XRP_ADDRESS.to_string(),
            amount: Some(PaymentAmount::ExactValue("10".to_string())),
            memo: Some("12345".to_string()),
            references: None,
            asset_id: Some(AssetId::from_chain(Chain::Xrp)),
        });

        assert_eq!(decode(Some(Chain::Xrp), &format!("{XRP_ADDRESS}?dt=12345&amount=10")).unwrap(), payment);
        assert_eq!(
            decode(Some(Chain::Bitcoin), &format!("{BITCOIN_ADDRESS}?dt=12345")).unwrap(),
            Payment::Request(PaymentRequest {
                address: BITCOIN_ADDRESS.to_string(),
                asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
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
    }
}
