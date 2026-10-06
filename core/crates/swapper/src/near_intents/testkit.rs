use crate::{QuoteRequest, SwapperQuoteAsset, testkit::mock_quote};
use num_bigint::BigUint;
use primitives::asset_constants::{ETHEREUM_USDC_ASSET_ID, HYPERCORE_SPOT_USDC_ASSET_ID};

pub fn mock_hypercore_quote_request() -> QuoteRequest {
    let mut request = mock_quote(SwapperQuoteAsset::from(HYPERCORE_SPOT_USDC_ASSET_ID.clone()), SwapperQuoteAsset::from(ETHEREUM_USDC_ASSET_ID.clone()));
    request.from_asset.decimals = 8;
    request.to_asset.decimals = 6;
    request.value = BigUint::from(10002430590_u64);
    request.options.use_max_amount = true;
    request
}
