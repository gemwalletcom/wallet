use crate::{RpcClient, SwapperError, route_cache::DiscoveryCache};
use alloy_primitives::{Address, B256};
use alloy_sol_types::SolCall;
use gem_evm::{
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    uniswap::{
        FeeTier,
        contracts::{v3::IUniswapV3Factory, v4::IUniswapV4StateView},
        path::TokenPair,
    },
};
use gem_jsonrpc::{client::JsonRpcClient, types::JsonRpcResults};
use primitives::{Chain, decode_hex};
use std::collections::BTreeSet;

#[derive(Debug, Default)]
pub(super) struct PoolDiscovery {
    cache: DiscoveryCache<FeeTier, FeeTier, (Chain, Address)>,
}

impl PoolDiscovery {
    pub(super) fn missing_pools(&self, chain: Chain, pairs: &[(Address, Address)], fee_tiers: &[FeeTier]) -> Vec<TokenPair> {
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

    pub(super) fn record_pools(&self, chain: Chain, discovered: &[(TokenPair, bool)]) {
        discovered.iter().for_each(|(pair, exists)| {
            self.cache
                .record_discovery((chain, pair.token_in), (chain, pair.token_out), [(pair.fee_tier, exists.then_some(pair.fee_tier))]);
        });
    }

    pub(super) fn path_may_exist(&self, chain: Chain, pairs: &[TokenPair]) -> bool {
        pairs.iter().all(|pair| {
            let probes = std::slice::from_ref(&pair.fee_tier);
            let exists = self
                .cache
                .candidates_for_probes((chain, pair.token_in), (chain, pair.token_out), probes)
                .contains(&pair.fee_tier);
            exists || !self.cache.missing_probes((chain, pair.token_in), (chain, pair.token_out), probes).is_empty()
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

pub(super) async fn discover_v3_pools(client: &JsonRpcClient<RpcClient>, factory: &str, pools: &[TokenPair]) -> Result<Vec<(TokenPair, bool)>, SwapperError> {
    let requests = pools.iter().map(|pool| {
        let data = IUniswapV3Factory::getPoolCall {
            tokenA: pool.token_in,
            tokenB: pool.token_out,
            fee: pool.fee_tier.as_u24(),
        }
        .abi_encode();
        EthereumRpc::Call {
            transaction: TransactionObject::new_call(factory, data),
            block: BlockParameter::Latest,
        }
    });
    let responses = client.batch_request::<String, _>(requests.collect()).await?;
    Ok(pools
        .iter()
        .zip(responses)
        .filter_map(|(pool, response)| {
            let result = response.take().ok()?;
            let bytes = decode_hex(&result).ok()?;
            let address = IUniswapV3Factory::getPoolCall::abi_decode_returns(&bytes).ok()?;
            Some((pool.clone(), address != Address::ZERO))
        })
        .collect())
}

pub(super) async fn discover_v4_pools(client: &JsonRpcClient<RpcClient>, state_view: &str, pools: &[(TokenPair, B256)]) -> Result<Vec<(TokenPair, bool)>, SwapperError> {
    let requests = pools.iter().map(|(_, pool_id)| {
        let data = IUniswapV4StateView::getSlot0Call { poolId: *pool_id }.abi_encode();
        EthereumRpc::Call {
            transaction: TransactionObject::new_call(state_view, data),
            block: BlockParameter::Latest,
        }
    });
    let responses = client.batch_request::<String, _>(requests.collect()).await?;
    Ok(decode_v4_discoveries(pools, responses))
}

fn decode_v4_discoveries(pools: &[(TokenPair, B256)], responses: JsonRpcResults<String>) -> Vec<(TokenPair, bool)> {
    pools
        .iter()
        .zip(responses)
        .filter_map(|((pool, _), response)| {
            let result = response.take().ok()?;
            let bytes = decode_hex(&result).ok()?;
            let slot = IUniswapV4StateView::getSlot0Call::abi_decode_returns(&bytes).ok()?;
            Some((pool.clone(), !slot.sqrtPriceX96.is_zero()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::U256;
    use alloy_sol_types::SolValue;
    use gem_jsonrpc::types::{JsonRpcError, JsonRpcErrorResponse, JsonRpcResponse, JsonRpcResult};
    use primitives::hex::encode_with_0x;

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
        assert!(discovery.path_may_exist(Chain::Ethereum, &[pools[0].clone()]));
        assert!(!discovery.path_may_exist(Chain::Ethereum, &[pools[1].clone()]));
        assert!(discovery.path_may_exist(Chain::Ethereum, &[pools[2].clone()]));
    }

    #[test]
    fn test_v4_discovery_preserves_successful_responses() {
        let pools = [
            (
                TokenPair {
                    token_in: Address::from([1; 20]),
                    token_out: Address::from([2; 20]),
                    fee_tier: FeeTier::FiveHundred,
                },
                B256::ZERO,
            ),
            (
                TokenPair {
                    token_in: Address::from([1; 20]),
                    token_out: Address::from([3; 20]),
                    fee_tier: FeeTier::ThreeThousand,
                },
                B256::ZERO,
            ),
        ];
        let slot = (U256::from(1), 0i32, 0u32, 0u32).abi_encode();
        let responses = vec![
            JsonRpcResult::Value(JsonRpcResponse {
                id: Some(1),
                result: encode_with_0x(&slot),
            }),
            JsonRpcResult::Error(JsonRpcErrorResponse {
                id: Some(2),
                error: JsonRpcError {
                    code: -32000,
                    message: "upstream unavailable".into(),
                    cause: None,
                },
            }),
        ]
        .into();

        let discovered = decode_v4_discoveries(&pools, responses);
        let discovery = PoolDiscovery::default();
        discovery.record_pools(Chain::Ethereum, &discovered);

        assert_eq!(discovered, vec![(pools[0].0.clone(), true)]);
        assert_eq!(
            discovery.missing_pools(Chain::Ethereum, &[(pools[0].0.token_in, pools[0].0.token_out)], std::slice::from_ref(&pools[0].0.fee_tier)),
            vec![]
        );
        assert_eq!(
            discovery.missing_pools(Chain::Ethereum, &[(pools[1].0.token_in, pools[1].0.token_out)], std::slice::from_ref(&pools[1].0.fee_tier)),
            vec![pools[1].0.clone()]
        );
    }
}
