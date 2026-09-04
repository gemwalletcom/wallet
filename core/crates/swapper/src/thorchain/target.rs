use gem_client::{Target, build_path_with_query};

use super::{THORChainNetwork, model::QuoteSwapRequest};

#[derive(Clone, Debug)]
pub enum ThorChainTarget {
    Quote { network: THORChainNetwork, request: QuoteSwapRequest },
    InboundAddresses { network: THORChainNetwork },
    AsgardVaults { network: THORChainNetwork },
    TransactionStatus { network: THORChainNetwork, hash: String },
}

impl Target for ThorChainTarget {
    fn path(&self) -> String {
        match self {
            Self::Quote { network, request } => build_path_with_query(&format!("/{network}/quote/swap"), request),
            Self::InboundAddresses { network } => format!("/{network}/inbound_addresses"),
            Self::AsgardVaults { network } => format!("/{network}/vaults/asgard"),
            Self::TransactionStatus { network, hash } => format!("/{network}/tx/status/{hash}"),
        }
    }
}
