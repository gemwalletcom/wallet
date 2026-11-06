use std::collections::HashSet;

use gem_encoding::decode_base64_url;
use primitives::{
    Chain,
    payment::{Payment, PaymentRequest},
};

use super::bip21;
use super::error::{PaymentDecoderError, Result};
use super::query;

const QUERY_ADDRESS: &str = "address";

pub fn decode(path: &str) -> Result<Payment> {
    let (path_address, query) = path.split_once('?').unwrap_or((path, ""));
    let parameters = query::parameters(query);
    if parameters
        .iter()
        .any(|(key, _)| key.rsplit_once('.').is_some_and(|(_, index)| !index.is_empty() && index.bytes().all(|byte| byte.is_ascii_digit())))
    {
        return Err(PaymentDecoderError::InvalidFormat("Multiple Zcash payments are not supported".to_string()));
    }
    let mut keys = HashSet::new();
    for (key, _) in &parameters {
        if !keys.insert(key) {
            return Err(PaymentDecoderError::InvalidFormat(format!("Duplicate Zcash parameter: {key}")));
        }
    }
    let address = match (path_address, query::values(&parameters, QUERY_ADDRESS).as_slice()) {
        (address, []) => address.to_string(),
        ("", [address]) => address.clone(),
        _ => return Err(PaymentDecoderError::InvalidFormat(format!("Duplicate Zcash parameter: {QUERY_ADDRESS}"))),
    };
    if !address.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
        return Err(PaymentDecoderError::InvalidFormat("Invalid Zcash address".to_string()));
    }
    let request = bip21::request(Some(Chain::Zcash), &address, &parameters)?;
    Ok(Payment::Request {
        request: PaymentRequest {
            memo: request
                .memo
                .as_deref()
                .map(|memo| {
                    decode_base64_url(memo)
                        .ok()
                        .and_then(|bytes| String::from_utf8(bytes).ok())
                        .ok_or_else(|| PaymentDecoderError::InvalidFormat("Invalid Zcash memo".to_string()))
                })
                .transpose()?,
            ..request
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, payment::PaymentAmount, testkit::zcash_mock::TEST_ZCASH_UNIFIED_ADDRESS as ADDRESS};

    #[test]
    fn test_decode() {
        assert_eq!(
            decode(&format!("{ADDRESS}?amount=0.5&memo=VGhhbmsgeW91&label=Coffee")).unwrap(),
            Payment::Request {
                request: PaymentRequest {
                    address: ADDRESS.to_string(),
                    amount: Some(PaymentAmount::ExactValue { value: "0.5".to_string() }),
                    memo: Some("Thank you".to_string()),
                    label: Some("Coffee".to_string()),
                    references: None,
                    asset_id: Some(AssetId::from_chain(Chain::Zcash)),
                },
            }
        );
        assert_eq!(
            decode(&format!("?address={ADDRESS}&amount=0.5&memo=VGhhbmsgeW91&label=Coffee")).unwrap(),
            decode(&format!("{ADDRESS}?amount=0.5&memo=VGhhbmsgeW91&label=Coffee")).unwrap()
        );
        assert_eq!(decode(&format!("{ADDRESS}?memo=not+base64")).unwrap_err(), PaymentDecoderError::InvalidFormat("Invalid Zcash memo".to_string()));
        assert_eq!(decode(&format!("{ADDRESS}?memo=_w")).unwrap_err(), PaymentDecoderError::InvalidFormat("Invalid Zcash memo".to_string()));
    }

    #[test]
    fn test_decode_rejects_ambiguous_payments() {
        let multiple = PaymentDecoderError::InvalidFormat("Multiple Zcash payments are not supported".to_string());
        let duplicate = |key: &str| PaymentDecoderError::InvalidFormat(format!("Duplicate Zcash parameter: {key}"));

        assert_eq!(decode(&format!("{ADDRESS}?amount=0.5&address.1={ADDRESS}&amount.1=0.1")).unwrap_err(), multiple);
        assert_eq!(decode(&format!("?address={ADDRESS}&amount.2=0.1")).unwrap_err(), multiple);
        assert_eq!(decode(&format!("{ADDRESS}?address={ADDRESS}")).unwrap_err(), duplicate("address"));
        assert_eq!(decode(&format!("?address={ADDRESS}&address={ADDRESS}")).unwrap_err(), duplicate("address"));
        assert_eq!(decode(&format!("?address={ADDRESS}&amount=0.5&amount=1")).unwrap_err(), duplicate("amount"));
        assert_eq!(decode(&format!("{ADDRESS}?memo=Zmlyc3Q&MEMO=c2Vjb25k")).unwrap_err(), duplicate("memo"));
        assert_eq!(
            decode(&format!("?address={ADDRESS}%3Famount%3D10%26label%3Dx&amount=0.5")).unwrap_err(),
            PaymentDecoderError::InvalidFormat("Invalid Zcash address".to_string())
        );
        assert_eq!(decode("?amount=0.5").unwrap_err(), PaymentDecoderError::MissingField("address".to_string()));
    }
}
