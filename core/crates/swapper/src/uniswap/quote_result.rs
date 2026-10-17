use crate::{RpcClient, SwapperError, route_cache::OutpricedCache};
use alloy_primitives::{Address, U256};
use gem_evm::{jsonrpc::EthereumRpc, uniswap::path::TokenPair};
use gem_jsonrpc::{
    client::JsonRpcClient,
    types::{JsonRpcResponse, JsonRpcResult, JsonRpcResults},
};
use primitives::Chain;

const EXECUTION_REVERTED: &str = "execution reverted";

pub type OutpricedPaths = OutpricedCache<Vec<TokenPair>, (Chain, Address)>;
pub type PathQuote = (Vec<TokenPair>, EthereumRpc);

#[derive(Debug, PartialEq)]
pub struct QuoteResult {
    pub amount_out: U256,
    pub pairs: Vec<TokenPair>,
}

impl QuoteResult {
    pub fn fee_tier(&self) -> Option<u32> {
        self.pairs.first().map(|pair| pair.fee_tier as u32)
    }

    pub fn intermediary(&self) -> Option<Address> {
        self.pairs.split_last().and_then(|(_, hops)| hops.first()).map(|pair| pair.token_out)
    }
}

pub async fn get_best_quote<F>(client: &JsonRpcClient<RpcClient>, outpriced: &OutpricedPaths, from: (Chain, Address), to: (Chain, Address), amount_in: u128, paths: Vec<PathQuote>, decoder: F) -> Result<QuoteResult, SwapperError>
where
    F: Fn(&JsonRpcResponse<String>) -> Result<(U256, U256), SwapperError>,
{
    outpriced
        .quote_live_first((&from, &to), amount_in, paths, |(pairs, _)| pairs.clone(), |paths| quote_paths(client, outpriced, (&from, &to), amount_in, paths, &decoder))
        .await
}

async fn quote_paths<F>(client: &JsonRpcClient<RpcClient>, outpriced: &OutpricedPaths, (from, to): (&(Chain, Address), &(Chain, Address)), amount_in: u128, paths: Vec<PathQuote>, decoder: &F) -> Result<QuoteResult, SwapperError>
where
    F: Fn(&JsonRpcResponse<String>) -> Result<(U256, U256), SwapperError>,
{
    let (pairs, calls): (Vec<_>, Vec<_>) = paths.into_iter().unzip();
    let results: JsonRpcResults<String> = client.requests(calls).await?;
    let outcomes = path_outcomes(pairs, results, decoder);
    outpriced.record(from, to, amount_in, outcomes.iter().map(|(pairs, amount_out)| (pairs.clone(), amount_out.map_or(0, |amount_out| amount_out.saturating_to()))));
    outcomes
        .into_iter()
        .filter_map(|(pairs, amount_out)| Some(QuoteResult { amount_out: amount_out?, pairs }))
        .max_by_key(|quote| quote.amount_out)
        .ok_or(SwapperError::NoQuoteAvailable)
}

fn path_outcomes<F>(pairs: Vec<Vec<TokenPair>>, results: JsonRpcResults<String>, decoder: &F) -> Vec<(Vec<TokenPair>, Option<U256>)>
where
    F: Fn(&JsonRpcResponse<String>) -> Result<(U256, U256), SwapperError>,
{
    pairs
        .into_iter()
        .zip(results.0)
        .filter_map(|(pairs, result)| match result {
            JsonRpcResult::Value(value) => Some((pairs, decoder(&value).ok().map(|(amount_out, _)| amount_out))),
            JsonRpcResult::Error(response) if response.error.message.starts_with(EXECUTION_REVERTED) => Some((pairs, None)),
            JsonRpcResult::Error(_) => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alien::mock::{MockFn, ProviderMock};
    use gem_evm::{
        jsonrpc::{BlockParameter, TransactionObject},
        uniswap::FeeTier,
    };
    use gem_jsonrpc::alien::create_client;
    use serde_json::{Value, json};
    use std::{
        str::FromStr,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };

    const TOKEN_IN: Address = Address::repeat_byte(1);
    const INTERMEDIARY: Address = Address::repeat_byte(2);
    const TOKEN_OUT: Address = Address::repeat_byte(3);

    fn mock_quoter(calls: Arc<AtomicUsize>) -> JsonRpcClient<RpcClient> {
        let response = move |request: &Value| match request["params"][0]["data"].as_str() {
            Some("0x01") => json!({"jsonrpc": "2.0", "id": request["id"], "result": "1000"}),
            Some("0x02") => json!({"jsonrpc": "2.0", "id": request["id"], "result": "975"}),
            _ => json!({"jsonrpc": "2.0", "id": request["id"], "error": {"code": 3, "message": EXECUTION_REVERTED}}),
        };
        let provider = ProviderMock {
            response: MockFn(Box::new(move |target| {
                let body = serde_json::from_slice::<Value>(&target.body.unwrap_or_default()).unwrap();
                let requests = body.as_array().cloned().unwrap_or_else(|| vec![body.clone()]);
                calls.fetch_add(requests.len(), Ordering::Relaxed);
                let responses = requests.iter().map(response).collect::<Vec<_>>();
                if body.is_array() { json!(responses) } else { responses[0].clone() }.to_string()
            })),
        };
        create_client(Arc::new(provider), Chain::Ethereum).unwrap()
    }

    fn path_quote(fee_tier: FeeTier, data: u8) -> PathQuote {
        let pairs = vec![TokenPair {
            token_in: TOKEN_IN,
            token_out: TOKEN_OUT,
            fee_tier,
        }];
        let call = EthereumRpc::Call {
            transaction: TransactionObject::new_call("0x0000000000000000000000000000000000000004", vec![data]),
            block: BlockParameter::Latest,
        };
        (pairs, call)
    }

    #[tokio::test]
    async fn test_get_best_quote_skips_outpriced_paths() {
        let calls = Arc::new(AtomicUsize::new(0));
        let client = mock_quoter(calls.clone());
        let outpriced = OutpricedPaths::default();
        let paths = || vec![path_quote(FeeTier::Hundred, 3), path_quote(FeeTier::FiveHundred, 1), path_quote(FeeTier::ThreeThousand, 2)];
        let decoder = |response: &JsonRpcResponse<String>| Ok((U256::from_str(&response.result).unwrap(), U256::ZERO));
        let quote = |amount_in| get_best_quote(&client, &outpriced, (Chain::Ethereum, TOKEN_IN), (Chain::Ethereum, TOKEN_OUT), amount_in, paths(), decoder);

        let best = quote(100).await.unwrap();
        assert_eq!((best.amount_out, best.fee_tier()), (U256::from(1000), Some(500)));
        assert_eq!(calls.swap(0, Ordering::Relaxed), 3);

        assert_eq!(quote(100).await.unwrap(), best);
        assert_eq!(calls.swap(0, Ordering::Relaxed), 2);

        assert_eq!(quote(50).await.unwrap(), best);
        assert_eq!(calls.swap(0, Ordering::Relaxed), 3);
    }

    #[test]
    fn test_quote_result_route() {
        let direct = QuoteResult {
            amount_out: U256::ZERO,
            pairs: vec![TokenPair {
                token_in: TOKEN_IN,
                token_out: TOKEN_OUT,
                fee_tier: FeeTier::ThreeThousand,
            }],
        };
        let two_hop = QuoteResult {
            amount_out: U256::ZERO,
            pairs: TokenPair::new_two_hop(&TOKEN_IN, &INTERMEDIARY, &TOKEN_OUT, FeeTier::FiveHundred),
        };

        assert_eq!((direct.fee_tier(), direct.intermediary()), (Some(3000), None));
        assert_eq!((two_hop.fee_tier(), two_hop.intermediary()), (Some(500), Some(INTERMEDIARY)));
    }
}
