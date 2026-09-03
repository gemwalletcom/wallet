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
