use super::error::{PaymentDecoderError, Result};
use crate::url_query::query_parameters;
use crate::{
    Chain,
    asset_id::AssetId,
    payment::{Payment, PaymentLink, PaymentRequest},
};
use url::{Url, form_urlencoded};
pub const SOLANA_PAY_SCHEME: &str = "solana";

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
    pub reference: Option<Vec<String>>,
    pub label: Option<String>,
    pub message: Option<String>,
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
            amount: val.amount,
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

    // Handle Transfer Request
    let (recipient, query) = query_part
        .split_once('?')
        .ok_or_else(|| PaymentDecoderError::InvalidFormat("Invalid URL query string".to_string()))?;

    let query_params = query_parameters(query);

    let amount = query_params.get("amount").cloned();
    let spl_token = query_params.get("spl-token").cloned();
    let reference = query_params.get("reference").map(|v| v.split(',').map(String::from).collect());
    let label = query_params.get("label").cloned();
    let message = query_params.get("message").cloned();
    let memo = query_params.get("memo").cloned();

    Ok(RequestType::Transfer(PayTransfer {
        recipient: recipient.to_string(),
        amount,
        spl_token,
        reference,
        label,
        message,
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
    fn test_parse_transfer() {
        let uri = format!(
            "mvines9iiHiQTysrwkJjGf2gb9Ex9jXJX8ns3qwf2kN?amount=1&spl-token={SOLANA_PAY_USDC_SPL_TOKEN}&reference=82ZJ7nbGpixjeDCmEhUcmwXYfvurzAgGdtSMuHnUgyny&label=Michael&message=Thanks%20for%20all%20the%20fish&memo=OrderId5678"
        );
        let pay_url = match parse(&uri).unwrap() {
            RequestType::Transfer(pay_url) => pay_url,
            _ => panic!("Wrong type"),
        };
        assert_eq!(pay_url.recipient, "mvines9iiHiQTysrwkJjGf2gb9Ex9jXJX8ns3qwf2kN");
        assert_eq!(pay_url.amount.unwrap(), "1");
        assert_eq!(pay_url.spl_token.unwrap(), SOLANA_PAY_USDC_SPL_TOKEN);
        assert_eq!(pay_url.reference.unwrap(), vec!["82ZJ7nbGpixjeDCmEhUcmwXYfvurzAgGdtSMuHnUgyny".to_string()]);
        assert_eq!(pay_url.label.unwrap(), "Michael");
        assert_eq!(pay_url.message.unwrap(), "Thanks for all the fish");
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
