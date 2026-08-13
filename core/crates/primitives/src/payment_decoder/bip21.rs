use std::str::FromStr;

use super::amount;
use super::error::{PaymentDecoderError, Result};
use super::query;
use crate::{
    Chain,
    asset_id::AssetId,
    payment::{Payment, PaymentRequest},
};

const DOGECOIN_SCHEME: &str = "dogecoin";
const RIPPLE_SCHEME: &str = "ripple";
const XRPL_SCHEME: &str = "xrpl";

const REQUIRED_PARAMETER_PREFIX: &str = "req-";
const AUTHORITY_PREFIX: &str = "//";
const QUERY_AMOUNT: &str = "amount";
const QUERY_MEMO: &str = "memo";
const QUERY_DESTINATION_TAG: &str = "dt";

pub fn decode(scheme: Option<&str>, path: &str) -> Result<Payment> {
    let asset_id = match scheme {
        Some(scheme) => Some(asset_id(scheme).ok_or(PaymentDecoderError::InvalidScheme)?),
        None => None,
    };
    let path = path.strip_prefix(AUTHORITY_PREFIX).unwrap_or(path);

    let Some((address, query)) = path.split_once('?') else {
        return Ok(Payment::Request(PaymentRequest {
            address: path.to_string(),
            amount: None,
            memo: None,
            asset_id,
        }));
    };

    let parameters = query::parameters(query);
    if let Some(required) = parameters.keys().find(|key| key.starts_with(REQUIRED_PARAMETER_PREFIX)) {
        return Err(PaymentDecoderError::InvalidFormat(format!("Unsupported required parameter: {required}")));
    }

    let is_xrp = asset_id.as_ref().is_some_and(|asset_id| asset_id.chain == Chain::Xrp);
    let memo = query::value(&parameters, QUERY_MEMO).or_else(|| query::value(&parameters, QUERY_DESTINATION_TAG).filter(|_| is_xrp));

    let amount = query::value(&parameters, QUERY_AMOUNT).as_deref().and_then(|value| match asset_id.as_ref() {
        Some(asset_id) => amount::from_coins(value, asset_id.chain),
        None => amount::normalize(value),
    });

    Ok(Payment::Request(PaymentRequest {
        address: address.to_string(),
        amount,
        memo,
        asset_id,
    }))
}

fn asset_id(scheme: &str) -> Option<AssetId> {
    chain(scheme).map(|chain| AssetId::from(chain, None))
}

fn chain(scheme: &str) -> Option<Chain> {
    match scheme {
        DOGECOIN_SCHEME => Some(Chain::Doge),
        RIPPLE_SCHEME | XRPL_SCHEME => Some(Chain::Xrp),
        _ => Chain::from_str(scheme).ok(),
    }
}
