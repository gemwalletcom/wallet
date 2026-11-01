use super::ChainflipRouteData;
use super::broker::{AssetsResponse, ChainflipAsset, DepositChannelRequest, QuoteRequest, RefundParameters};
use super::destination::VaultSwapDestination;
use super::provider::refund_parameters;
use super::refund::VaultSwapRefund;
use crate::fees::DEFAULT_CHAINFLIP_FEE_BPS;
use crate::{Quote, Route};
use primitives::known_assets::ARBITRUM_USDC;
use primitives::testkit::signer_mock::{TEST_BITCOIN_SENDER, TEST_EVM_RECIPIENT, TEST_EVM_SENDER, TEST_POLKADOT_SENDER, TEST_SOLANA_SENDER, TEST_TRON_SENDER};
use primitives::{AssetId, Chain};

pub const TRON_NOTE: &str = "0x07037e5f4552091a69125d5dfcb7b8c2659029395bdf00890101960000003e1451cdb84d440345de6195b0384d1b77aa4eaa0000000000000000000000000000000000000000000000000000000000000000000000001e83d2972d3dca3a330d60c2777ee5b8d25683c63fa359116985609830f42054050004002d";
pub const ETHEREUM_NATIVE_MIN_PRICE: &str = "0x3292bb84399604d6d7d2344";
pub const SOLANA_MIN_PRICE: &str = "0x80b80fe579baecccccccccccccccc";

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
    VaultSwapDestination::new(&AssetId::from_chain(Chain::Bitcoin), TEST_BITCOIN_SENDER).unwrap()
}

pub fn arbitrum_usdc_destination() -> VaultSwapDestination {
    VaultSwapDestination::new(&ARBITRUM_USDC.id, TEST_EVM_SENDER).unwrap()
}

pub fn mock_refund(chain: Chain, address: &str, min_price: &str) -> VaultSwapRefund {
    VaultSwapRefund::new(chain, refund_parameters(address, min_price))
}

pub fn ethereum_native_refund() -> VaultSwapRefund {
    mock_refund(Chain::Ethereum, TEST_EVM_SENDER, ETHEREUM_NATIVE_MIN_PRICE)
}

pub fn ethereum_usdc_refund() -> VaultSwapRefund {
    mock_refund(Chain::Ethereum, TEST_EVM_SENDER, "0x4faa1adcd75c490000000000000000")
}

pub fn solana_refund() -> VaultSwapRefund {
    mock_refund(Chain::Solana, TEST_SOLANA_SENDER, SOLANA_MIN_PRICE)
}

pub fn tron_refund() -> VaultSwapRefund {
    mock_refund(Chain::Tron, TEST_TRON_SENDER, "0x0")
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
