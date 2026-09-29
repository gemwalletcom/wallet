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
    Actions { network: THORChainNetwork, query: MidgardActionsQuery },
}

impl Target for MidgardTarget {
    fn path(&self) -> String {
        match self {
            Self::Actions { network, query } => build_path_with_query(&format!("{}/actions", network.midgard_path()), query),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midgard_actions_path() {
        let target = |network| MidgardTarget::Actions {
            network,
            query: MidgardActionsQuery {
                affiliate: "g1".to_string(),
                action_type: "swap",
                limit: 50,
                from_timestamp: None,
                next_page_token: None,
            },
        };

        assert_eq!(target(THORChainNetwork::Thorchain).path(), "/chain/thorchain_midgard/v2/actions?affiliate=g1&type=swap&limit=50");
        assert_eq!(target(THORChainNetwork::Mayachain).path(), "/v2/actions?affiliate=g1&type=swap&limit=50");
    }
}
