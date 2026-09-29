use gem_client::{Target, build_path_with_query};

use super::model::{GetSwapEvmParams, GetSwapSolanaParams, MayanSwapsQuery, QuoteQuery};

#[derive(Clone, Debug)]
pub enum MayanTarget {
    Quote { query: QuoteQuery },
    Chains,
    TransactionStatus { hash: String },
    SwapEvm { params: GetSwapEvmParams },
    SwapSolana { params: GetSwapSolanaParams },
    SwapSui,
    Swaps { query: MayanSwapsQuery },
}

impl Target for MayanTarget {
    fn path(&self) -> String {
        match self {
            Self::Quote { query } => build_path_with_query("/v3/quote", query),
            Self::Chains => "/v3/chains".to_string(),
            Self::TransactionStatus { hash } => format!("/v3/swap/trx/{hash}"),
            Self::SwapEvm { params } => build_path_with_query("/v3/get-swap/evm", params),
            Self::SwapSolana { params } => build_path_with_query("/v3/get-swap/solana", params),
            Self::SwapSui => "/v3/get-swap/sui".to_string(),
            Self::Swaps { query } => build_path_with_query("/v3/swaps", query),
        }
    }
}
