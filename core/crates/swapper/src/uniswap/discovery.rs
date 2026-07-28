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
    cache: RouteCache<FeeTier, FeeTier, (Chain, Address)>,
}

impl PoolDiscovery {
    pub fn missing_pools(&self, chain: Chain, pairs: &[(Address, Address)], fee_tiers: &[FeeTier]) -> Vec<TokenPair> {
        pairs
            .iter()
            .flat_map(|(token_in, token_out)| {
                self.cache
                    .missing_probes((chain, *token_in), (chain, *token_out), fee_tiers)
                    .into_iter()
                    .map(|fee_tier| TokenPair {
                        token_in: *token_in,
                        token_out: *token_out,
                        fee_tier,
                    })
            })
            .collect()
    }

    pub fn record_pools(&self, chain: Chain, discovered: &[(TokenPair, bool)]) {
        discovered.iter().for_each(|(pair, exists)| {
            self.cache
                .record_discovery((chain, pair.token_in), (chain, pair.token_out), [(pair.fee_tier, exists.then_some(pair.fee_tier))]);
        });
    }

    pub fn path_exists(&self, chain: Chain, pairs: &[TokenPair]) -> bool {
        pairs
            .iter()
            .all(|pair| self.cache.get_discovery((chain, pair.token_in), (chain, pair.token_out)).0.contains(&pair.fee_tier))
    }
}

pub(super) fn candidate_pairs(token_in: Address, token_out: Address, intermediaries: Vec<Address>) -> Vec<(Address, Address)> {
    std::iter::once((token_in, token_out))
        .chain(intermediaries.into_iter().flat_map(|intermediary| [(token_in, intermediary), (intermediary, token_out)]))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) async fn discover_v3_pools(client: &JsonRpcClient<RpcClient>, factory: &str, pools: &[TokenPair]) -> Result<Vec<(TokenPair, bool)>, SwapperError> {
    let requests = pools.iter().map(|pool| {
        let data = IUniswapV3Factory::getPoolCall {
            tokenA: pool.token_in,
            tokenB: pool.token_out,
            fee: pool.fee_tier.as_u24(),
        }
        .abi_encode();
        EthereumRpc::Call(TransactionObject::new_call(factory, data), BlockParameter::Latest)
    });
    let responses = client.batch_request::<_, String>(requests.collect()).await?;
    pools
        .iter()
        .zip(responses)
        .map(|(pool, response)| {
            let result = response.take()?;
            let bytes = hex_decode(result)?;
            let address = IUniswapV3Factory::getPoolCall::abi_decode_returns(&bytes)?;
            Ok((pool.clone(), address != Address::ZERO))
        })
        .collect()
}

pub(super) async fn discover_v4_pools(client: &JsonRpcClient<RpcClient>, state_view: &str, pools: &[(TokenPair, B256)]) -> Result<Vec<(TokenPair, bool)>, SwapperError> {
    let requests = pools.iter().map(|(_, pool_id)| {
        let data = IUniswapV4StateView::getSlot0Call { poolId: *pool_id }.abi_encode();
        EthereumRpc::Call(TransactionObject::new_call(state_view, data), BlockParameter::Latest)
    });
    let responses = client.batch_request::<_, String>(requests.collect()).await?;
    pools
        .iter()
        .zip(responses)
        .map(|((pool, _), response)| match response {
            JsonRpcResult::Value(response) => {
                let bytes = hex_decode(response.result)?;
                let slot = IUniswapV4StateView::getSlot0Call::abi_decode_returns(&bytes)?;
                Ok((pool.clone(), !slot.sqrtPriceX96.is_zero()))
            }
            JsonRpcResult::Error(error) => Err(SwapperError::from(error.error)),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_discovery() {
        let token_a = Address::from([1; 20]);
        let token_b = Address::from([2; 20]);
        let token_c = Address::from([3; 20]);
        let pairs = vec![(token_a, token_b), (token_a, token_c)];
        let fee_tiers = vec![FeeTier::FiveHundred, FeeTier::ThreeThousand];
        let discovery = PoolDiscovery::default();

        let pools = discovery.missing_pools(Chain::Ethereum, &pairs, &fee_tiers);
        assert_eq!(pools.len(), 4);

        discovery.record_pools(Chain::Ethereum, &[(pools[0].clone(), true), (pools[1].clone(), false)]);

        assert_eq!(discovery.missing_pools(Chain::Ethereum, &pairs, &fee_tiers), pools[2..]);
        assert_eq!(discovery.path_exists(Chain::Ethereum, &[pools[0].clone()]), true);
        assert_eq!(discovery.path_exists(Chain::Ethereum, &[pools[1].clone()]), false);
    }
}
