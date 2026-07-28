use crate::{RpcClient, SwapperError, route_cache::RouteCache};
use alloy_primitives::{Address, B256, hex::decode as hex_decode};
use alloy_sol_types::SolCall;
use gem_evm::{
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    uniswap::{
        FeeTier,
        contracts::{v3::IUniswapV3Factory, v4::IUniswapV4StateView},
        path::TokenPair,
    },
};
use gem_jsonrpc::{client::JsonRpcClient, types::JsonRpcResult};
use primitives::Chain;
use std::collections::BTreeSet;

#[derive(Debug, Default)]
pub(super) struct PoolDiscovery {
    cache: RouteCache<FeeTier, FeeTier>,
}

impl PoolDiscovery {
    fn asset_key(chain: Chain, token: Address) -> String {
        format!("{}:{}", chain, token)
    }

    pub fn missing_fee_tiers(&self, chain: Chain, token_a: Address, token_b: Address, fee_tiers: &[FeeTier]) -> Vec<FeeTier> {
        let from = Self::asset_key(chain, token_a);
        let to = Self::asset_key(chain, token_b);
        self.cache.missing_probes(&from, &to, fee_tiers)
    }

    pub fn record_fee_tiers(&self, chain: Chain, token_a: Address, token_b: Address, discovered: &[(FeeTier, bool)]) {
        let from = Self::asset_key(chain, token_a);
        let to = Self::asset_key(chain, token_b);
        self.cache
            .record_discovery(&from, &to, discovered.iter().map(|(tier, exists)| (*tier, exists.then_some(*tier))));
    }

    pub fn path_exists(&self, chain: Chain, pairs: &[TokenPair]) -> bool {
        pairs.iter().all(|pair| {
            let from = Self::asset_key(chain, pair.token_in);
            let to = Self::asset_key(chain, pair.token_out);
            self.cache.has_candidate(&from, &to, &pair.fee_tier)
        })
    }
}

pub(super) fn candidate_pairs(token_in: Address, token_out: Address, intermediaries: Vec<Address>) -> Vec<(Address, Address)> {
    std::iter::once((token_in, token_out))
        .chain(intermediaries.into_iter().flat_map(|intermediary| [(token_in, intermediary), (intermediary, token_out)]))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) async fn discover_v3_pools(
    client: &JsonRpcClient<RpcClient>,
    factory: &str,
    token_a: Address,
    token_b: Address,
    fee_tiers: &[FeeTier],
) -> Result<Vec<(FeeTier, bool)>, SwapperError> {
    let requests = fee_tiers.iter().map(|fee_tier| {
        let data = IUniswapV3Factory::getPoolCall {
            tokenA: token_a,
            tokenB: token_b,
            fee: fee_tier.as_u24(),
        }
        .abi_encode();
        EthereumRpc::Call(TransactionObject::new_call(factory, data), BlockParameter::Latest)
    });
    let responses = client.batch_request::<_, String>(requests.collect()).await?;
    fee_tiers
        .iter()
        .zip(responses)
        .map(|(fee_tier, response)| {
            let result = response.take()?;
            let bytes = hex_decode(result)?;
            let pool = IUniswapV3Factory::getPoolCall::abi_decode_returns(&bytes)?;
            Ok((*fee_tier, pool != Address::ZERO))
        })
        .collect()
}

pub(super) async fn discover_v4_pools(client: &JsonRpcClient<RpcClient>, state_view: &str, pools: &[(FeeTier, B256)]) -> Result<Vec<(FeeTier, bool)>, SwapperError> {
    let requests = pools.iter().map(|(_, pool_id)| {
        let data = IUniswapV4StateView::getSlot0Call { poolId: *pool_id }.abi_encode();
        EthereumRpc::Call(TransactionObject::new_call(state_view, data), BlockParameter::Latest)
    });
    let responses = client.batch_request::<_, String>(requests.collect()).await?;
    pools
        .iter()
        .zip(responses)
        .map(|((fee_tier, _), response)| match response {
            JsonRpcResult::Value(response) => {
                let bytes = hex_decode(response.result)?;
                let slot = IUniswapV4StateView::getSlot0Call::abi_decode_returns(&bytes)?;
                Ok((*fee_tier, !slot.sqrtPriceX96.is_zero()))
            }
            JsonRpcResult::Error(error) => Err(SwapperError::from(error.error)),
        })
        .collect()
}
