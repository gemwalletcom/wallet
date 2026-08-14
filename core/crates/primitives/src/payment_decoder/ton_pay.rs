use super::amount;
use super::error::{PaymentDecoderError, Result};
use super::query;
use crate::{
    AssetId, Chain,
    payment::{Payment, PaymentAmount, PaymentRequest},
};

pub const TON_PAY_SCHEME: &str = "ton";
const TRANSFER_PATH: &str = "transfer";

const QUERY_AMOUNT: &str = "amount";
const QUERY_TEXT: &str = "text";
const QUERY_BODY: &str = "bin";
const QUERY_STATE_INIT: &str = "init";

pub fn decode(path: &str) -> Result<Payment> {
    let (path, query) = path.split_once('?').unwrap_or((path, ""));
    let parameters = query::parameters(query);
    query::reject_unsupported(&parameters, &[QUERY_BODY, QUERY_STATE_INIT])?;

    Ok(Payment::Request(PaymentRequest {
        address: address(path)?,
        amount: query::value(&parameters, QUERY_AMOUNT)
            .and_then(|value| amount::exact_from_atomic(&value, Chain::Ton))
            .map(PaymentAmount::ExactValue),
        memo: query::value(&parameters, QUERY_TEXT),
        asset_id: Some(AssetId::from_chain(Chain::Ton)),
    }))
}

fn address(path: &str) -> Result<String> {
    let path = path.trim_matches('/');

    match path.split_once('/') {
        None => Ok(path.to_string()),
        Some((TRANSFER_PATH, address)) if !address.contains('/') => Ok(address.to_string()),
        Some(_) => Err(PaymentDecoderError::InvalidFormat(format!("Not a transfer path: {path}"))),
    }
}
