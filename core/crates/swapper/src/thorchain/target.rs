use gem_client::{Target, build_path_with_query};

use super::model::QuoteSwapRequest;

#[derive(Clone, Debug)]
pub enum ThorChainTarget {
    Quote(QuoteSwapRequest),
    InboundAddresses,
    AsgardVaults,
    TransactionStatus(String),
}

impl Target for ThorChainTarget {
    fn path(&self) -> String {
        match self {
            Self::Quote(request) => build_path_with_query("/thorchain/quote/swap", request),
            Self::InboundAddresses => "/thorchain/inbound_addresses".to_string(),
            Self::AsgardVaults => "/thorchain/vaults/asgard".to_string(),
            Self::TransactionStatus(hash) => format!("/thorchain/tx/status/{hash}"),
        }
    }
}
