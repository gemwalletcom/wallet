use super::ActionResponse;
use crate::{Options, QuoteRequest, SwapperQuoteAsset};
use num_bigint::BigUint;
use primitives::{AssetId, Chain};

impl QuoteRequest {
    pub fn mock_cosmos_to_stellar() -> Self {
        QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Cosmos)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Stellar)),
            wallet_address: "COSMOS_ADDRESS".into(),
            destination_address: "STELLAR_ADDRESS".into(),
            value: BigUint::from(1000000u64),
            options: Options::default(),
        }
    }
}

impl ActionResponse {
    pub fn mock() -> Self {
        serde_json::from_str(include_str!("testdata/action_response.json")).unwrap()
    }
}
