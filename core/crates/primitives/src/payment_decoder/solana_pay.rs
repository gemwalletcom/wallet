use super::amount;
use super::error::{PaymentDecoderError, Result};
use super::query;
use crate::{
    Chain,
    asset_id::AssetId,
    payment::{Payment, PaymentAmount, PaymentLink, PaymentRequest},
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

    let token = query::value(&parameters, QUERY_SPL_TOKEN);
    let amount = query::value(&parameters, QUERY_AMOUNT)
        .and_then(|value| match &token {
            Some(_) => amount::normalize(&value),
            None => amount::exact(&value, Chain::Solana),
        })
        .map(PaymentAmount::ExactValue);

    Ok(Payment::Request(PaymentRequest {
        address: recipient.to_string(),
        amount,
        memo: query::value(&parameters, QUERY_MEMO),
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
