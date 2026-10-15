use super::ActionResponse;
use crate::{Options, ProviderData, ProviderType, Quote, QuoteRequest, Route, SwapperProvider, SwapperQuoteAsset};
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

    pub fn mock_sui_to_ton() -> Self {
        QuoteRequest {
            from_asset: SwapperQuoteAsset::from(Chain::Sui.as_asset_id()),
            to_asset: SwapperQuoteAsset::from(Chain::Ton.as_asset_id()),
            wallet_address: "SUI_ADDRESS".into(),
            destination_address: "TON_ADDRESS".into(),
            value: BigUint::from(3_379_105_015_u64),
            options: Options::default(),
        }
    }
}

impl ActionResponse {
    pub fn mock() -> Self {
        serde_json::from_str(include_str!("testdata/action_response.json")).unwrap()
    }

    pub fn mock_sui_to_ton() -> Self {
        serde_json::from_str(include_str!("testdata/action_sui_to_ton.json")).unwrap()
    }
}

impl Quote {
    pub fn mock_swaps_xyz(request: QuoteRequest, response: ActionResponse) -> Self {
        Quote {
            from_value: response.amount_in.amount.clone(),
            min_from_value: None,
            to_value: response.amount_out.amount.clone(),
            data: ProviderData {
                provider: ProviderType::new(SwapperProvider::SwapsXyz),
                slippage_bps: request.options.slippage.bps,
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&response).unwrap(),
                }],
            },
            request,
            eta_in_seconds: None,
        }
    }
}
