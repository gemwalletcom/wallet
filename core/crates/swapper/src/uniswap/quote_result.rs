use crate::SwapperError;
use alloy_primitives::U256;
use gem_jsonrpc::types::{JsonRpcResponse, JsonRpcResult, JsonRpcResults};

#[derive(Debug, Clone, Copy)]
pub struct QuotePosition {
    pub route_idx: usize,
    pub fee_tier_idx: usize,
}

#[derive(Debug)]
pub struct QuoteResult {
    pub amount_out: U256,
    pub route_idx: usize,
    pub fee_tier_idx: usize,
}

pub fn get_best_quote<F>(results: &JsonRpcResults<String>, positions: &[QuotePosition], decoder: F) -> Result<QuoteResult, SwapperError>
where
    F: Fn(&JsonRpcResponse<String>) -> Result<(U256, U256), SwapperError>,
{
    results
        .0
        .iter()
        .zip(positions)
        .filter_map(|(result, position)| match result {
            JsonRpcResult::Value(value) => decoder(value).ok().map(|quote| QuoteResult {
                amount_out: quote.0,
                route_idx: position.route_idx,
                fee_tier_idx: position.fee_tier_idx,
            }),
            JsonRpcResult::Error(_) => None,
        })
        .max_by_key(|quote| quote.amount_out)
        .ok_or(SwapperError::NoQuoteAvailable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_jsonrpc::types::{JsonRpcError, JsonRpcErrorResponse};

    #[test]
    fn test_get_best_quote_preserves_position() {
        let results = vec![
            JsonRpcResult::Value(JsonRpcResponse {
                id: Some(1),
                result: "10".to_string(),
            }),
            JsonRpcResult::Error(JsonRpcErrorResponse {
                id: Some(2),
                error: JsonRpcError {
                    code: -1,
                    message: "reverted".to_string(),
                },
            }),
            JsonRpcResult::Value(JsonRpcResponse {
                id: Some(3),
                result: "30".to_string(),
            }),
        ]
        .into();
        let positions = [
            QuotePosition { route_idx: 0, fee_tier_idx: 0 },
            QuotePosition { route_idx: 1, fee_tier_idx: 0 },
            QuotePosition { route_idx: 1, fee_tier_idx: 1 },
        ];

        let quote = get_best_quote(&results, &positions, |response| Ok((U256::from(response.result.parse::<u64>().unwrap()), U256::ZERO))).unwrap();

        assert_eq!(quote.amount_out, U256::from(30));
        assert_eq!(quote.route_idx, 1);
        assert_eq!(quote.fee_tier_idx, 1);
    }
}
