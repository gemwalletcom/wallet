use super::ChainflipRouteData;
use super::broker::{AssetsResponse, ChainflipAsset, DepositChannelRequest, QuoteRequest, RefundParameters};
use super::destination::VaultSwapDestination;
use crate::fees::DEFAULT_CHAINFLIP_FEE_BPS;
use crate::{Quote, Route};
use primitives::known_assets::ARBITRUM_USDC;
use primitives::testkit::signer_mock::{TEST_EVM_RECIPIENT, TEST_POLKADOT_SENDER};
use primitives::{AssetId, Chain};

pub const TRON_NOTE: &str = "0x0703514bcb1f9aabb904e6106bd1052b66d2706dbbb70089010196000000eca9bc828a3005b9a3b909f2cc5c2a54794de05f0000000000000000000000000000000000000000000000000000000000000000000000001e83d2972d3dca3a330d60c2777ee5b8d25683c63fa359116985609830f42054050004002d";
pub const BITCOIN_DESTINATION: &str = "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq";
pub const ARBITRUM_DESTINATION: &str = "0x514bcb1f9aabb904e6106bd1052b66d2706dbbb7";

impl AssetsResponse {
    pub fn mock() -> Self {
        serde_json::from_str(include_str!("./broker/test/assets.json")).unwrap()
    }
}

impl QuoteRequest {
    pub fn mock(amount: &str, source_asset: &str, destination_asset: &str) -> Self {
        Self {
            amount: amount.parse().unwrap(),
            source_asset: source_asset.to_string(),
            destination_asset: destination_asset.to_string(),
            commission_bps: DEFAULT_CHAINFLIP_FEE_BPS,
            is_vault_swap: true,
        }
    }
}

impl DepositChannelRequest {
    pub fn mock() -> Self {
        Self {
            source_asset: ChainflipAsset {
                chain: "Assethub".to_string(),
                asset: "DOT".to_string(),
            },
            destination_asset: ChainflipAsset {
                chain: "Ethereum".to_string(),
                asset: "ETH".to_string(),
            },
            destination_address: TEST_EVM_RECIPIENT.to_string(),
            boost_fee: None,
            refund_parameters: RefundParameters {
                retry_duration: 150,
                refund_address: TEST_POLKADOT_SENDER.to_string(),
                min_price: "0x1".to_string(),
                max_oracle_price_slippage: None,
            },
            dca_parameters: None,
        }
    }
}

pub fn bitcoin_destination() -> VaultSwapDestination {
    VaultSwapDestination::new(&AssetId::from_chain(Chain::Bitcoin), BITCOIN_DESTINATION).unwrap()
}

pub fn arbitrum_usdc_destination() -> VaultSwapDestination {
    VaultSwapDestination::new(&ARBITRUM_USDC.id, ARBITRUM_DESTINATION).unwrap()
}

pub fn mock_routes(quote: &Quote, estimated_price: f64) -> Vec<Route> {
    let route_data = ChainflipRouteData {
        boost_fee: None,
        estimated_price,
        dca_parameters: None,
    };
    vec![Route {
        input: quote.request.from_asset.asset_id(),
        output: quote.request.to_asset.asset_id(),
        route_data: serde_json::to_string(&route_data).unwrap(),
    }]
}
