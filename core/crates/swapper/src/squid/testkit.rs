use crate::{Options, QuoteRequest, SwapperQuoteAsset};
use num_bigint::BigUint;
use primitives::{AssetId, Chain};

pub const TEST_OSMOSIS_ADDRESS: &str = "osmo1tkvyjqeq204rmrrz3w4hcrs336qahsfwn8m0ye";
pub const TEST_COSMOS_ADDRESS: &str = "cosmos1tkvyjqeq204rmrrz3w4hcrs336qahsfwmugljt";

impl QuoteRequest {
    pub fn mock_osmosis_to_cosmos() -> Self {
        QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Osmosis)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Cosmos)),
            wallet_address: TEST_OSMOSIS_ADDRESS.to_string(),
            destination_address: TEST_COSMOS_ADDRESS.to_string(),
            value: BigUint::from(10000000u64),
            options: Options::new_with_slippage(100.into()),
        }
    }
}
