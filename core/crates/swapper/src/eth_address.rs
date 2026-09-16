use alloy_primitives::Address;

use super::error::{INVALID_ADDRESS, SwapperError};
use primitives::AssetId;

pub(crate) fn parse_asset_id(asset: &AssetId) -> Result<Address, SwapperError> {
    if let Some(token_id) = &asset.token_id {
        parse_str(token_id)
    } else {
        Err(SwapperError::ComputeQuoteError(format!("{}: {}", INVALID_ADDRESS, asset)))
    }
}

pub(crate) fn parse_str(str: &str) -> Result<Address, SwapperError> {
    str.parse::<Address>().map_err(|_| SwapperError::ComputeQuoteError(format!("{}: {}", INVALID_ADDRESS, str)))
}
