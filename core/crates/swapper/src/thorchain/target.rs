use gem_client::{Target, build_path_with_query};

use super::{
    THORChainNetwork,
    model::{MidgardActionsQuery, QuoteSwapRequest},
};

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

#[derive(Clone, Debug)]
pub enum MidgardTarget {
    Actions { query: MidgardActionsQuery },
}

impl Target for MidgardTarget {
    fn path(&self) -> String {
        match self {
            Self::Actions { query } => build_path_with_query("/v2/actions", query),
        }
    }
}
