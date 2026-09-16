use super::{DEFAULT_FILL_TIMEOUT, asset::parse_address};
use crate::{Options, ProviderData, ProviderType, Quote, QuoteRequest, Route, SwapperProvider, eth_address};
use alloy_primitives::{Address, Bytes, U256, hex::encode_prefixed as HexEncode};
use alloy_sol_types::SolValue;
use gem_evm::across::{contracts::V3SpokePoolInterface::V3RelayData, deployment::AcrossDeployment};
use num_bigint::BigUint;
use primitives::{
    Chain,
    asset_constants::{ETHEREUM_USDT_ASSET_ID, ETHEREUM_USDT_TOKEN_ID, TRON_USDT_ASSET_ID, TRON_USDT_TOKEN_ID},
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
    pub fn mock_across_tron() -> Self {
        let request = QuoteRequest {
            from_asset: TRON_USDT_ASSET_ID.clone().into(),
            to_asset: ETHEREUM_USDT_ASSET_ID.clone().into(),
            wallet_address: "TJRyWwFs9wTFGZg3JbrVriFbNfCug5tDeC".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: BigUint::from(10000000u64),
            options: Options::default(),
        };
        Quote {
            from_value: request.value.clone(),
            min_from_value: None,
            to_value: BigUint::from(9990000u64),
            data: ProviderData {
                provider: ProviderType::new(SwapperProvider::Across),
                slippage_bps: request.options.slippage.bps,
                routes: vec![Route {
                    input: TRON_USDT_ASSET_ID.clone(),
                    output: ETHEREUM_USDT_ASSET_ID.clone(),
                    route_data: HexEncode(mock_v3_relay_data().abi_encode()),
                }],
            },
            request,
            eta_in_seconds: Some(120),
        }
    }
}
