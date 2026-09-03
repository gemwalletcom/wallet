use gem_client::{Target, build_path_with_query};

use super::model::{QuoteParams, SwapParams};

#[derive(Clone, Debug)]
pub(super) enum OkxTarget {
    Quote { params: QuoteParams },
    Swap { params: SwapParams },
}

impl Target for OkxTarget {
    fn path(&self) -> String {
        match self {
            Self::Quote { params } => build_path_with_query("/api/v6/dex/aggregator/quote", params),
            Self::Swap { params } => build_path_with_query("/api/v6/dex/aggregator/swap", params),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        assert_eq!(
            OkxTarget::Quote {
                params: QuoteParams {
                    chain_index: "1".into(),
                    amount: "1000".into(),
                    from_token_address: "0xfrom".into(),
                    to_token_address: "0xto".into(),
                    slippage_percent: "0.5".into(),
                    dex_ids: None,
                    fee_percent: "0.1".into(),
                }
            }
            .path(),
            "/api/v6/dex/aggregator/quote?chainIndex=1&amount=1000&fromTokenAddress=0xfrom&toTokenAddress=0xto&slippagePercent=0.5&feePercent=0.1"
        );
    }
}
