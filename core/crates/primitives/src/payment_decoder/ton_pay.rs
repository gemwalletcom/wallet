use super::error::{PaymentDecoderError, Result};
use std::collections::HashMap;
use url::form_urlencoded;

use crate::{AssetId, Chain};

pub const TON_PAY_SCHEME: &str = "ton";
pub const TON_PAY_TYPE_TRANSFER: &str = "transfer";

const QUERY_AMOUNT: &str = "amount";
const QUERY_TEXT: &str = "text";

#[derive(Debug, Clone)]
pub struct TonPayment {
    pub recipient: String,
    pub asset_id: AssetId,
    pub amount: Option<String>,
    pub comment: Option<String>,
}

pub fn parse(uri: &str) -> Result<TonPayment> {
    let scheme = format!("{TON_PAY_SCHEME}:");
    if !uri.starts_with(&scheme) {
        return Err(PaymentDecoderError::InvalidScheme);
    }
    let remainder = &uri[scheme.len()..];
    let (path, query) = remainder.split_once('?').unwrap_or((remainder, ""));
    let recipient = extract_address(path)?;
    let parameters: HashMap<String, String> = form_urlencoded::parse(query.as_bytes())
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();

    Ok(TonPayment {
        recipient,
        asset_id: AssetId::from_chain(Chain::Ton),
        amount: parameters.get(QUERY_AMOUNT).cloned(),
        comment: parameters.get(QUERY_TEXT).cloned(),
    })
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
        let uri = "ton://transfer/UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA";
        let payment = parse(uri).unwrap();
        assert_eq!(payment.recipient, "UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA");
        assert_eq!(payment.amount, None);
        assert_eq!(payment.comment, None);
    }

    #[test]
    fn test_parse_without_transfer() {
        let uri = "ton://UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA";
        let payment = parse(uri).unwrap();
        assert_eq!(payment.recipient, "UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA");
    }

    #[test]
    fn test_parse_with_query() {
        let uri = "ton://transfer/UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA?amount=1000000000&text=hello+world";
        let payment = parse(uri).unwrap();
        assert_eq!(payment.recipient, "UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA");
        assert_eq!(payment.amount, Some("1000000000".to_string()));
        assert_eq!(payment.comment, Some("hello world".to_string()));
    }

    #[test]
    fn test_parse_invalid_uri() {
        let uri = "ton://invalid/format";
        assert!(parse(uri).is_err());
    }
}
