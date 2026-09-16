pub use primitives::testkit::signer_mock::TEST_TON_SENDER;

use super::{
    constants::FALLBACK_ROUTERS,
    model::{Router, SwapSimulation},
    provider::Stonfi,
    quote::{DiscoveredPool, PoolData, router_model},
    tx_builder::{ReferralParams, SwapTransactionParams},
};
use gem_client::testkit::MockClient;
use gem_ton::{
    address::Address,
    constants::TON_PROXY_JETTON_ADDRESS,
    models::{RunGetMethodResult, StackEntry},
    rpc::client::TonClient,
};
use num_bigint::BigUint;
use primitives::asset_constants::TON_USDT_TOKEN_ID;

#[cfg(all(test, feature = "swap_integration_tests"))]
pub const NOT_TOKEN_ID: &str = "EQAvlWFDxGF2lXm67y4yzC17wYKD9A0guwPkMs1gOsM__NOT";
pub const ROUTER_V2_ADDRESS: &str = "EQCS4UEa5UaJLzOyyKieqQOQ2P9M-7kXpkO5HnP3Bv250cN3";
pub const TEST_PTON_WALLET: &str = "EQCSIMGBps_qzRG3uPYhON8bucyCtu0mYdL1-u4gSz77IBa3";
pub const TEST_USDT_WALLET: &str = "EQCSLWJ9fY7b0A5OI72wxUp27l4fRlc6GvRBeFf6PiPpH4p3";

impl SwapSimulation {
    pub fn mock(offer_jetton_wallet: &str, ask_jetton_wallet: &str, ask_units: &str, min_ask_units: &str) -> Self {
        Self {
            offer_jetton_wallet: offer_jetton_wallet.to_string(),
            ask_jetton_wallet: ask_jetton_wallet.to_string(),
            router: Router {
                address: ROUTER_V2_ADDRESS.to_string(),
                major_version: 2,
                minor_version: 2,
            },
            ask_units: ask_units.to_string(),
            min_ask_units: min_ask_units.to_string(),
        }
    }
}

impl<'a> SwapTransactionParams<'a> {
    pub fn mock(simulation: &'a SwapSimulation) -> Self {
        Self {
            simulation,
            next_swap: None,
            from_native: true,
            to_native: false,
            sender_jetton_wallet: None,
            from_value: "1000000000",
            wallet_address: Address::parse(TEST_TON_SENDER).unwrap(),
            receiver_address: Address::parse(TEST_TON_SENDER).unwrap(),
            referral: ReferralParams {
                address: Address::parse(TEST_TON_SENDER).unwrap(),
                bps: 50,
            },
            deadline: Some(1_700_000_000),
        }
    }
}

impl PoolData {
    pub fn mock() -> Self {
        Self {
            is_locked: false,
            reserve0: BigUint::from(3_809_436_784_065u64),
            reserve1: BigUint::from(1_784_561_670_122_756u64),
            token0_wallet: TEST_USDT_WALLET.to_string(),
            token1_wallet: TEST_PTON_WALLET.to_string(),
            lp_fee: 7,
            protocol_fee: 3,
        }
    }
}

impl DiscoveredPool {
    pub fn mock(pool_address: &str) -> Self {
        Self {
            pool_address: pool_address.to_string(),
            router: router_model(&FALLBACK_ROUTERS[0]),
            asset0: TON_PROXY_JETTON_ADDRESS.to_string(),
            asset1: TON_USDT_TOKEN_ID.to_string(),
            wallet0: TEST_PTON_WALLET.to_string(),
            wallet1: TEST_USDT_WALLET.to_string(),
            lp_fee_bps: None,
        }
    }
}

impl Stonfi<MockClient> {
    pub fn mock<F>(handler: F) -> Self
    where
        F: Fn(&str, &str) -> Vec<u8> + Send + Sync + 'static,
    {
        Self::new_with_client(TonClient::new(MockClient::new().with_post(move |_, body| {
            let request: serde_json::Value = serde_json::from_slice(body).unwrap();
            let address = request["address"].as_str().unwrap();
            let method = request["method"].as_str().unwrap();
            Ok(handler(method, address))
        })))
    }
}

pub fn mock_cell_response(address: &str) -> Vec<u8> {
    mock_get_method_response(vec![StackEntry::Cell(Address::parse(address).unwrap().to_boc_base64().unwrap())])
}

pub fn mock_pool_data_response(is_locked: bool, reserve0: u64, reserve1: u64, token0_wallet: &str, token1_wallet: &str, lp_fee_bps: u32) -> Vec<u8> {
    let token0 = Address::parse(token0_wallet).unwrap().to_boc_base64().unwrap();
    let token1 = Address::parse(token1_wallet).unwrap().to_boc_base64().unwrap();
    mock_get_method_response(vec![
        StackEntry::Num(if is_locked { "0x1" } else { "0x0" }.into()),
        StackEntry::Num("0x0".into()),
        StackEntry::Num("0x0".into()),
        StackEntry::Num(reserve0.to_string()),
        StackEntry::Num(reserve1.to_string()),
        StackEntry::Cell(token0),
        StackEntry::Cell(token1.clone()),
        StackEntry::Num(lp_fee_bps.to_string()),
        StackEntry::Num("0x3".into()),
        StackEntry::Num("0x0".into()),
        StackEntry::Num("0x0".into()),
        StackEntry::Cell(token1),
    ])
}

pub fn mock_v1_pool_data_response(reserve0: u64, reserve1: u64, token0_wallet: &str, token1_wallet: &str, lp_fee_bps: u32) -> Vec<u8> {
    let token0 = Address::parse(token0_wallet).unwrap().to_boc_base64().unwrap();
    let token1 = Address::parse(token1_wallet).unwrap().to_boc_base64().unwrap();
    mock_get_method_response(vec![
        StackEntry::Num(reserve0.to_string()),
        StackEntry::Num(reserve1.to_string()),
        StackEntry::Cell(token0),
        StackEntry::Cell(token1.clone()),
        StackEntry::Num(lp_fee_bps.to_string()),
        StackEntry::Num("0xa".into()),
        StackEntry::Num("0xa".into()),
        StackEntry::Cell(token1),
        StackEntry::Num("0x0".into()),
        StackEntry::Num("0x0".into()),
    ])
}

fn mock_get_method_response(stack: Vec<StackEntry>) -> Vec<u8> {
    serde_json::to_vec(&RunGetMethodResult { exit_code: 0, stack }).unwrap()
}
