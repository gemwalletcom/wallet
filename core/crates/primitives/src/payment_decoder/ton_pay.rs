use super::amount::from_smallest_unit;
use super::error::{PaymentDecoderError, Result};
use super::query;

use crate::{
    AssetId, Chain,
    payment::{Payment, PaymentRequest},
};

pub const TON_PAY_SCHEME: &str = "ton";
const TON_PAY_TYPE_TRANSFER: &str = "transfer";

const QUERY_AMOUNT: &str = "amount";
const QUERY_TEXT: &str = "text";
const QUERY_BODY: &str = "bin";
const QUERY_STATE_INIT: &str = "init";

#[derive(Debug, Clone)]
pub struct TonPayment {
    pub recipient: String,
    pub amount: Option<String>,
    pub comment: Option<String>,
}

pub fn parse(path: &str) -> Result<TonPayment> {
    let (path, query) = path.split_once('?').unwrap_or((path, ""));
    let recipient = extract_address(path)?;
    let parameters = query::parameters(query);
    query::reject_unsupported(&parameters, &[QUERY_BODY, QUERY_STATE_INIT])?;

    Ok(TonPayment {
        recipient,
        amount: query::value(&parameters, QUERY_AMOUNT),
        comment: query::value(&parameters, QUERY_TEXT),
    })
}

pub fn decode(path: &str) -> Result<Payment> {
    let payment = parse(path)?;
    Ok(Payment::Request(PaymentRequest {
        address: payment.recipient,
        amount: payment.amount.and_then(|amount| from_smallest_unit(&amount, Chain::Ton)),
        memo: payment.comment,
        asset_id: Some(AssetId::from_chain(Chain::Ton)),
    }))
}

fn extract_address(path: &str) -> Result<String> {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() == 2 && parts[0] == TON_PAY_TYPE_TRANSFER {
        Ok(parts[1].to_string())
    } else if parts.len() == 1 {
        Ok(parts[0].to_string())
    } else {
        Err(PaymentDecoderError::InvalidFormat(format!("Invalid URI format: {}", path)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_transfer() {
        let uri = "//transfer/UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA";
        let payment = parse(uri).unwrap();
        assert_eq!(payment.recipient, "UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA");
        assert_eq!(payment.amount, None);
        assert_eq!(payment.comment, None);
    }

    #[test]
    fn test_parse_without_transfer() {
        let uri = "//UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA";
        let payment = parse(uri).unwrap();
        assert_eq!(payment.recipient, "UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA");
    }

    #[test]
    fn test_parse_with_query() {
        let uri = "//transfer/UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA?amount=1000000000&text=hello+world";
        let payment = parse(uri).unwrap();
        assert_eq!(payment.recipient, "UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA");
        assert_eq!(payment.amount, Some("1000000000".to_string()));
        assert_eq!(payment.comment, Some("hello world".to_string()));
    }

    #[test]
    fn test_parse_invalid_uri() {
        let uri = "//invalid/format";
        assert!(parse(uri).is_err());
    }
}
