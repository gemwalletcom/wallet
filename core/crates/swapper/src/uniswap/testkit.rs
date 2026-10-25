use alloy_primitives::{Address, U256, aliases::U112, hex};
use alloy_sol_types::SolCall;
use gem_evm::uniswap::contracts::{v2::IUniswapV2Pair, v3::IUniswapV3Factory};
use primitives::hex::encode_with_0x;

use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

use super::routed_asset::{Funding, RoutedAsset};
use crate::{
    alien::mock::{MockFn, ProviderMock},
    eth_address,
};

pub type Posts = Arc<Mutex<Vec<usize>>>;

impl RoutedAsset {
    pub fn mock(address: Address, funding: Funding, scale: u64) -> Self {
        Self { address, scale: U256::from(scale), funding }
    }

    pub fn mock_token(token_id: &str) -> Self {
        Self::mock(eth_address::parse_str(token_id).unwrap(), Funding::Token, 1)
    }

    pub fn mock_value(token_id: &str) -> Self {
        Self::mock(eth_address::parse_str(token_id).unwrap(), Funding::Value, 1)
    }

    pub fn mock_router_balance(token_id: &str, scale: u64) -> Self {
        Self::mock(eth_address::parse_str(token_id).unwrap(), Funding::RouterBalance, scale)
    }
}

pub fn mock_reserves_result(reserve0: u128, reserve1: u128) -> String {
    encode_with_0x(&IUniswapV2Pair::getReservesCall::abi_encode_returns(&IUniswapV2Pair::getReservesReturn {
        reserve0: U112::from(reserve0),
        reserve1: U112::from(reserve1),
        blockTimestampLast: 0,
    }))
}

pub fn mock_batch_provider(respond: impl Fn(&Value) -> Value + Send + Sync + 'static) -> (ProviderMock, Posts) {
    let posts = Posts::default();
    let recorded = posts.clone();
    let provider = ProviderMock {
        response: MockFn(Box::new(move |target| {
            let body = serde_json::from_slice::<Value>(&target.body.unwrap_or_default()).unwrap();
            let requests = body.as_array().cloned().unwrap_or_else(|| vec![body.clone()]);
            recorded.lock().unwrap().push(requests.len());
            let responses = requests.iter().map(&respond).collect::<Vec<_>>();
            if body.is_array() { json!(responses) } else { responses[0].clone() }.to_string()
        })),
    };
    (provider, posts)
}

pub fn mock_taxed_token_provider(taxed: Address, respond: impl Fn(&str, &str) -> Option<String> + Send + Sync + 'static) -> (ProviderMock, Posts) {
    let no_pool = encode_with_0x(&IUniswapV3Factory::getPoolCall::abi_encode_returns(&Address::ZERO));
    mock_batch_provider(move |request| {
        let call = &request["params"][0];
        let data = call["data"].as_str().unwrap_or_default();
        let result = match request["params"].as_array().map(Vec::len) {
            Some(3) if data.contains(&hex::encode(taxed)) => return json!({"jsonrpc": "2.0", "id": request["id"], "error": {"code": 3, "message": "execution reverted"}}),
            Some(3) => "0x".to_string(),
            _ => respond(call["to"].as_str().unwrap_or_default(), data).unwrap_or_else(|| no_pool.clone()),
        };
        json!({"jsonrpc": "2.0", "id": request["id"], "result": result})
    })
}
