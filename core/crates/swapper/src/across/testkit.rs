use super::{DEFAULT_FILL_TIMEOUT, asset::parse_address};
use crate::{ProviderData, ProviderType, Quote, QuoteRequest, Route, SwapperProvider, eth_address, testkit::mock_quote};
use alloy_primitives::{Address, Bytes, U256, hex::encode_prefixed as HexEncode};
use alloy_sol_types::SolValue;
use gem_evm::{
    across::{asset::AcrossAsset, contracts::V3SpokePoolInterface::V3RelayData, deployment::AcrossDeployment},
    u256::u256_to_biguint,
};
use num_bigint::BigUint;
use primitives::{
    AssetId, Chain,
    asset_constants::{ARC_USDC_TOKEN_ID, BASE_USDC_ASSET_ID, ETHEREUM_USDT_ASSET_ID, ETHEREUM_USDT_TOKEN_ID, TRON_USDT_ASSET_ID, TRON_USDT_TOKEN_ID},
};

pub const TEST_FILL_DEADLINE: u32 = 1_700_000_000 + DEFAULT_FILL_TIMEOUT;

pub fn mock_v3_relay_data() -> V3RelayData {
    V3RelayData {
        depositor: parse_address(Chain::Tron, "TJRyWwFs9wTFGZg3JbrVriFbNfCug5tDeC").unwrap(),
        recipient: eth_address::parse_str("0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7").unwrap(),
        exclusiveRelayer: Address::ZERO,
        inputToken: parse_address(Chain::Tron, TRON_USDT_TOKEN_ID).unwrap(),
        outputToken: eth_address::parse_str(ETHEREUM_USDT_TOKEN_ID).unwrap(),
        inputAmount: U256::from(10_000_000),
        outputAmount: U256::from(9_990_000),
        originChainId: U256::from(AcrossDeployment::deployment_by_chain(&Chain::Tron).unwrap().chain_id),
        depositId: u32::MAX,
        fillDeadline: TEST_FILL_DEADLINE,
        exclusivityDeadline: 0,
        message: Bytes::new(),
    }
}

impl Quote {
    pub fn mock_across(request: QuoteRequest, relay_data: V3RelayData) -> Self {
        let routed = |asset_id: AssetId| AcrossAsset::from_asset(&asset_id).unwrap().asset_id;
        Quote {
            from_value: request.value.clone(),
            min_from_value: None,
            to_value: u256_to_biguint(&relay_data.outputAmount),
            data: ProviderData {
                provider: ProviderType::new(SwapperProvider::Across),
                slippage_bps: request.options.slippage.bps,
                routes: vec![Route {
                    input: routed(request.from_asset.asset_id()),
                    output: routed(request.to_asset.asset_id()),
                    route_data: HexEncode(relay_data.abi_encode()),
                }],
            },
            request,
            eta_in_seconds: None,
        }
    }

    pub fn mock_across_tron() -> Self {
        let request = QuoteRequest {
            wallet_address: "TJRyWwFs9wTFGZg3JbrVriFbNfCug5tDeC".to_string(),
            value: BigUint::from(10_000_000u64),
            ..mock_quote(TRON_USDT_ASSET_ID.clone().into(), ETHEREUM_USDT_ASSET_ID.clone().into())
        };
        Self::mock_across(request, mock_v3_relay_data())
    }

    pub fn mock_across_arc() -> Self {
        let request = QuoteRequest {
            value: BigUint::from(5_000_000_000_000_000_000u64),
            ..mock_quote(AssetId::from_chain(Chain::Arc).into(), BASE_USDC_ASSET_ID.clone().into())
        };
        let relay_data = V3RelayData {
            depositor: eth_address::parse_str(&request.wallet_address).unwrap(),
            inputToken: eth_address::parse_str(ARC_USDC_TOKEN_ID).unwrap(),
            inputAmount: U256::from(5_000_000),
            originChainId: U256::from(AcrossDeployment::deployment_by_chain(&Chain::Arc).unwrap().chain_id),
            ..mock_v3_relay_data()
        };
        Self::mock_across(request, relay_data)
    }
}
