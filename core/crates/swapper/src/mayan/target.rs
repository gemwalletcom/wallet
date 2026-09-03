use gem_client::{Target, build_path_with_query};

use super::model::{GetSwapEvmParams, GetSwapSolanaParams, QuoteQuery};

#[derive(Clone, Debug)]
pub enum MayanTarget {
    Quote { query: QuoteQuery },
    Chains,
    TransactionStatus { hash: String },
    SwapEvm { params: GetSwapEvmParams },
    SwapSolana { params: GetSwapSolanaParams },
    SwapSui,
}

impl Target for MayanTarget {
    fn path(&self) -> String {
        match self {
            Self::Quote { query } => build_path_with_query("/quote", query),
            Self::Chains => "/chains".to_string(),
            Self::TransactionStatus { hash } => format!("/swap/trx/{hash}"),
            Self::SwapEvm { params } => build_path_with_query("/get-swap/evm", params),
            Self::SwapSolana { params } => build_path_with_query("/get-swap/solana", params),
            Self::SwapSui => "/get-swap/sui".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use primitives::swap::SlippageMode;

    use super::*;
    use crate::mayan::model::QuoteParams;

    #[test]
    fn test_path() {
        let quote = MayanTarget::Quote {
            query: QuoteQuery::from(QuoteParams {
                amount_in64: "1000000".to_string(),
                from_token: "0x0000000000000000000000000000000000000000".to_string(),
                from_chain: "ethereum".to_string(),
                to_token: "So11111111111111111111111111111111111111112".to_string(),
                to_chain: "solana".to_string(),
                referrer: "0x1111111111111111111111111111111111111111".to_string(),
                referrer_bps: 50,
                slippage_bps: 100,
                slippage_mode: SlippageMode::Auto,
            }),
        };

        assert_eq!(
            quote.path(),
            "/quote?wormhole=false&swift=true&mctp=true&shuttle=false&fastMctp=true&gasless=false&onlyDirect=false&fullList=false&monoChain=true&solanaProgram=FC4eXxkyrMPTjiYUpp4EAnkmwMbQyZ6NDCh1kfLn6vsf&forwarderAddress=0x337685fdaB40D39bd02028545a4FfA7D287cC3E2&amountIn64=1000000&fromToken=0x0000000000000000000000000000000000000000&fromChain=ethereum&toToken=So11111111111111111111111111111111111111112&toChain=solana&referrer=0x1111111111111111111111111111111111111111&referrerBps=50&sdkVersion=14_1_0&slippageBps=auto"
        );
        assert_eq!(MayanTarget::TransactionStatus { hash: "0xabc".into() }.path(), "/swap/trx/0xabc");
    }
}
