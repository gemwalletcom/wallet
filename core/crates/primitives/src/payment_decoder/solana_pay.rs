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
const TRANSACTION_LINK_PREFIX: &str = "https";

const QUERY_AMOUNT: &str = "amount";
const QUERY_SPL_TOKEN: &str = "spl-token";
const QUERY_MEMO: &str = "memo";
const QUERY_REFERENCE: &str = "reference";

pub fn decode(path: &str) -> Result<Payment> {
    if path.starts_with(TRANSACTION_LINK_PREFIX) {
        return Ok(Payment::Link(PaymentLink::SolanaPay(transaction_link(path)?)));
    }

    let (recipient, query) = path.split_once('?').unwrap_or((path, ""));
    let parameters = query::parameters(query);
    query::reject_unsupported(&parameters, &[QUERY_REFERENCE])?;

    Ok(Payment::Request(PaymentRequest {
        address: recipient.to_string(),
        amount: query::value(&parameters, QUERY_AMOUNT).as_deref().and_then(amount::from_coins),
        memo: query::value(&parameters, QUERY_MEMO),
        asset_id: Some(AssetId::from(Chain::Solana, query::value(&parameters, QUERY_SPL_TOKEN))),
    }))
}

fn transaction_link(path: &str) -> Result<String> {
    let decoded = form_urlencoded::parse(format!("value={path}").as_bytes())
        .next()
        .map(|(_, value)| value.into_owned())
        .ok_or_else(|| PaymentDecoderError::InvalidFormat("Invalid percent encoding".to_string()))?;

    Ok(Url::parse(&decoded)?.to_string())
}
