use super::amount;
use super::error::{PaymentDecoderError, Result};
use super::query;
use crate::{
    Chain,
    asset_id::AssetId,
    payment::{Payment, PaymentLink, PaymentRequest},
};
use url::{Url, form_urlencoded};
pub const SOLANA_PAY_SCHEME: &str = "solana";

const QUERY_AMOUNT: &str = "amount";
const QUERY_SPL_TOKEN: &str = "spl-token";
const QUERY_MEMO: &str = "memo";
const QUERY_REFERENCE: &str = "reference";

#[derive(Debug, Clone)]
pub enum RequestType {
    Transfer(PayTransfer),
    Transaction(String),
}

#[derive(Debug, Clone)]
pub struct PayTransfer {
    pub recipient: String,
    pub amount: Option<String>,
    pub spl_token: Option<String>,
    pub memo: Option<String>,
}

pub fn decode(path: &str) -> Result<Payment> {
    match parse(path)? {
        RequestType::Transfer(transfer) => Ok(Payment::Request(transfer.into())),
        RequestType::Transaction(link) => Ok(Payment::Link(PaymentLink::SolanaPay(link))),
    }
}

impl From<PayTransfer> for PaymentRequest {
    fn from(val: PayTransfer) -> Self {
        Self {
            address: val.recipient,
            amount: val.amount.as_deref().and_then(amount::from_coins),
            memo: val.memo,
            asset_id: Some(AssetId::from(Chain::Solana, val.spl_token.map(|token| token.to_string()))),
        }
    }
}

pub fn parse(path: &str) -> Result<RequestType> {
    let query_part = path.to_string();
    if query_part.starts_with("https") {
        let encoded = format!("value={query_part}");
        let decoded = form_urlencoded::parse(encoded.as_bytes())
            .next()
            .map(|(_, v)| v.into_owned())
            .ok_or_else(|| PaymentDecoderError::InvalidFormat("Invalid percent encoding".to_string()))?;
        let url = Url::parse(&decoded)?;
        return Ok(RequestType::Transaction(url.to_string()));
    }

    let (recipient, query) = query_part.split_once('?').unwrap_or((&query_part, ""));

    let query_params = query::parameters(query);

    query::reject_unsupported(&query_params, &[QUERY_REFERENCE])?;

    let amount = query::value(&query_params, QUERY_AMOUNT);
    let spl_token = query::value(&query_params, QUERY_SPL_TOKEN);
    let memo = query::value(&query_params, QUERY_MEMO);

    Ok(RequestType::Transfer(PayTransfer {
        recipient: recipient.to_string(),
        amount,
        spl_token,
        memo,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOLANA_PAY_USDC_SPL_TOKEN: &str = crate::asset_constants::SOLANA_USDC_TOKEN_ID;

    #[test]
    fn test_parse_transaction_encoded_https() {
        let uri = "https%3A%2F%2Fmy.site%2Fpay%3Fcheckout%3D1";
        let link = match parse(uri).unwrap() {
            RequestType::Transaction(pay_url) => pay_url,
            _ => panic!("Wrong type"),
        };

        assert_eq!(link, "https://my.site/pay?checkout=1");
    }

    #[test]
    fn test_parse_transaction_plain_https() {
        let uri = "https://another.example/pay";
        let link = match parse(uri).unwrap() {
            RequestType::Transaction(pay_url) => pay_url,
            _ => panic!("Wrong type"),
        };

        assert_eq!(link, "https://another.example/pay");
    }

    #[test]
    fn test_parse_transfer_rejects_a_reference() {
        let uri = "mvines9iiHiQTysrwkJjGf2gb9Ex9jXJX8ns3qwf2kN?amount=1&reference=82ZJ7nbGpixjeDCmEhUcmwXYfvurzAgGdtSMuHnUgyny";
        assert!(parse(uri).is_err());
    }

    #[test]
    fn test_parse_transfer() {
        let uri = format!(
            "mvines9iiHiQTysrwkJjGf2gb9Ex9jXJX8ns3qwf2kN?amount=1&spl-token={SOLANA_PAY_USDC_SPL_TOKEN}&label=Michael&message=Thanks%20for%20all%20the%20fish&memo=OrderId5678"
        );
        let pay_url = match parse(&uri).unwrap() {
            RequestType::Transfer(pay_url) => pay_url,
            _ => panic!("Wrong type"),
        };
        assert_eq!(pay_url.recipient, "mvines9iiHiQTysrwkJjGf2gb9Ex9jXJX8ns3qwf2kN");
        assert_eq!(pay_url.amount.unwrap(), "1");
        assert_eq!(pay_url.spl_token.unwrap(), SOLANA_PAY_USDC_SPL_TOKEN);
        assert_eq!(pay_url.memo.unwrap(), "OrderId5678");
    }

    #[test]
    fn test_parse_transaction() {
        let uri = "https://example.com/solana-pay";
        let link = match parse(uri).unwrap() {
            RequestType::Transaction(pay_url) => pay_url,
            _ => panic!("Wrong type"),
        };

        assert_eq!(link, "https://example.com/solana-pay");

        let uri = "https%3A%2F%2Fexample.com%2Fsolana-pay%3Forder%3D12345";
        let link = match parse(uri).unwrap() {
            RequestType::Transaction(pay_url) => pay_url,
            _ => panic!("Wrong type"),
        };

        assert_eq!(link, "https://example.com/solana-pay?order=12345");
    }
}
