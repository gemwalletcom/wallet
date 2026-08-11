use std::str::FromStr;

use super::error::{PaymentDecoderError, Result};
use super::query;
use crate::{
    Chain,
    asset_id::AssetId,
    payment::{Payment, PaymentRequest},
};

const REQUIRED_PARAMETER_PREFIX: &str = "req-";
const QUERY_AMOUNT: &str = "amount";
const QUERY_MEMO: &str = "memo";
const XRP_SCHEMES: [&str; 2] = ["ripple", "xrpl"];
const QUERY_DESTINATION_TAG: &str = "dt";

pub fn decode(scheme: Option<&str>, path: &str) -> Result<Payment> {
    let asset_id = scheme.and_then(asset_id);

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
    let memo = parameters.get(QUERY_MEMO).or_else(|| parameters.get(QUERY_DESTINATION_TAG).filter(|_| is_xrp)).cloned();

    Ok(Payment::Request(PaymentRequest {
        address: address.to_string(),
        amount: parameters.get(QUERY_AMOUNT).cloned(),
        memo,
        asset_id,
    }))
}

fn asset_id(scheme: &str) -> Option<AssetId> {
    let chain = match scheme {
        scheme if XRP_SCHEMES.contains(&scheme) => Chain::Xrp,
        scheme => Chain::from_str(scheme).ok()?,
    };
    Some(AssetId::from(chain, None))
}
