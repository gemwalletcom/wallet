use super::amount;
use super::error::{PaymentDecoderError, Result};
use super::query;
use crate::{
    AssetId, Chain,
    payment::{Payment, PaymentAmount, PaymentLink, PaymentRequest},
};
use url::{Url, form_urlencoded};

const TRANSACTION_LINK_PREFIX: &str = "https";

const QUERY_AMOUNT: &str = "amount";
const QUERY_SPL_TOKEN: &str = "spl-token";
const QUERY_MEMO: &str = "memo";
const QUERY_REFERENCE: &str = "reference";

pub fn decode(path: &str) -> Result<Payment> {
    if path.starts_with(TRANSACTION_LINK_PREFIX) {
        return Ok(Payment::Link(PaymentLink::SolanaPay { url: transaction_link(path)? }));
    }

    let (recipient, query) = path.split_once('?').unwrap_or((path, ""));
    if recipient.is_empty() {
        return Err(PaymentDecoderError::MissingField("recipient".to_string()));
    }
    let parameters = query::parameters(query);

    let token = query::value(&parameters, QUERY_SPL_TOKEN);
    let amount = query::value(&parameters, QUERY_AMOUNT)
        .and_then(|value| match &token {
            Some(_) => amount::decimal(&value),
            None => amount::exact(&value, Chain::Solana),
        })
        .map(PaymentAmount::ExactValue);

    Ok(Payment::Request(PaymentRequest {
        address: recipient.to_string(),
        amount,
        memo: query::value(&parameters, QUERY_MEMO),
        references: match query::values(&parameters, QUERY_REFERENCE) {
            references if references.is_empty() => None,
            references => Some(references),
        },
        asset_id: Some(AssetId::from(Chain::Solana, token)),
    }))
}

fn transaction_link(path: &str) -> Result<String> {
    let decoded = form_urlencoded::parse(format!("value={path}").as_bytes())
        .next()
        .map(|(_, value)| value.into_owned())
        .ok_or_else(|| PaymentDecoderError::InvalidFormat("Invalid percent encoding".to_string()))?;

    Ok(Url::parse(&decoded)?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset_constants::SOLANA_USDC_TOKEN_ID;

    const RECIPIENT: &str = "HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5";

    #[test]
    fn test_decode() {
        assert_eq!(
            decode(RECIPIENT).unwrap(),
            Payment::Request(PaymentRequest {
                address: RECIPIENT.to_string(),
                asset_id: Some(AssetId::from_chain(Chain::Solana)),
                ..PaymentRequest::mock()
            })
        );
        assert_eq!(
            decode(&format!("{RECIPIENT}?amount=0.266232")).unwrap(),
            Payment::Request(PaymentRequest {
                address: RECIPIENT.to_string(),
                amount: Some(PaymentAmount::ExactValue("0.266232".to_string())),
                asset_id: Some(AssetId::from_chain(Chain::Solana)),
                ..PaymentRequest::mock()
            })
        );
        assert_eq!(
            decode(&format!("{RECIPIENT}?amount=1&spl-token={SOLANA_USDC_TOKEN_ID}&label=Michael&memo=OrderId5678")).unwrap(),
            Payment::Request(PaymentRequest {
                address: RECIPIENT.to_string(),
                amount: Some(PaymentAmount::ExactValue("1".to_string())),
                memo: Some("OrderId5678".to_string()),
                references: None,
                asset_id: Some(AssetId::from(Chain::Solana, Some(SOLANA_USDC_TOKEN_ID.to_string()))),
            })
        );
    }

    #[test]
    fn test_decode_transaction_link() {
        assert_eq!(
            decode("https://merchant.example/pay?order=12345").unwrap(),
            Payment::Link(PaymentLink::SolanaPay {
                url: "https://merchant.example/pay?order=12345".to_string()
            })
        );
        assert_eq!(
            decode("https%3A%2F%2Fapi.spherepay.co%2Fv1%2Fpublic%2FpaymentLink%2Fpay%2FpaymentLink_1%3Fnetwork%3Dsol").unwrap(),
            Payment::Link(PaymentLink::SolanaPay {
                url: "https://api.spherepay.co/v1/public/paymentLink/pay/paymentLink_1?network=sol".to_string()
            })
        );
    }

    #[test]
    fn test_decode_references() {
        let first = "82ZJ7nbGpixjeDCmEhUcmwXYfvurzAgGdtSMuHnUgyny";
        let second = "7GUcQZQwHHa9GBPhVq7v2LArSsp5VmGXV5zXnQ8Q7N3a";
        assert_eq!(
            decode(&format!("{RECIPIENT}?amount=1&reference={first}&reference={second}")),
            Ok(Payment::Request(PaymentRequest {
                address: RECIPIENT.to_string(),
                amount: Some(PaymentAmount::ExactValue("1".to_string())),
                references: Some(vec![first.to_string(), second.to_string()]),
                asset_id: Some(AssetId::from_chain(Chain::Solana)),
                ..PaymentRequest::mock()
            }))
        );
    }
}
