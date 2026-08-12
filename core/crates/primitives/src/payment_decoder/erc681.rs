use super::amount;
use super::error::{PaymentDecoderError, Result};
use super::query;
use crate::{
    Chain,
    asset_id::AssetId,
    payment::{Payment, PaymentRequest},
};

pub const ETHEREUM_SCHEME: &str = "ethereum";
const PAY_PREFIX: &str = "pay-";
const TRANSFER_FUNCTION: &str = "transfer";

const QUERY_ADDRESS: &str = "address";
const QUERY_AMOUNT: &str = "amount";
const QUERY_MEMO: &str = "memo";
const QUERY_VALUE: &str = "value";

pub fn decode(path: &str) -> Result<Payment> {
    let (path, query) = path.split_once('?').unwrap_or((path, ""));
    let (target, function) = path.split_once('/').map_or((path, None), |(target, function)| (target, Some(function)));
    let (target, chain) = match target.split_once('@') {
        Some((target, network_id)) => (target, Chain::from_network_id(network_id).unwrap_or(Chain::Ethereum)),
        None => (target, Chain::Ethereum),
    };
    let target = target.strip_prefix(PAY_PREFIX).unwrap_or(target);
    let parameters = query::parameters(query);
    let memo = query::value(&parameters, QUERY_MEMO);

    match function {
        Some(TRANSFER_FUNCTION) => Ok(Payment::Request(PaymentRequest {
            address: query::value(&parameters, QUERY_ADDRESS).ok_or_else(|| PaymentDecoderError::MissingField(QUERY_ADDRESS.to_string()))?,
            amount: None,
            memo,
            asset_id: Some(AssetId::from(chain, Some(target.to_string()))),
        })),
        Some(function) => Err(PaymentDecoderError::InvalidFormat(format!("Unsupported function: {function}"))),
        None => Ok(Payment::Request(PaymentRequest {
            address: target.to_string(),
            amount: query::value(&parameters, QUERY_VALUE)
                .as_deref()
                .and_then(|value| amount::from_smallest_unit(value, chain))
                .or_else(|| query::value(&parameters, QUERY_AMOUNT).as_deref().and_then(amount::from_coins)),
            memo,
            asset_id: Some(AssetId::from(chain, None)),
        })),
    }
}
