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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        assert_eq!(
            ThorChainTarget::Quote {
                network: THORChainNetwork::Thorchain,
                request: QuoteSwapRequest {
                    from_asset: "BTC.BTC".into(),
                    to_asset: "ETH.ETH".into(),
                    amount: "100000000".into(),
                    affiliate: "gem".into(),
                    affiliate_bps: 50,
                    streaming_interval: 1,
                    streaming_quantity: 0,
                },
            }
            .path(),
            "/thorchain/quote/swap?from_asset=BTC.BTC&to_asset=ETH.ETH&amount=100000000&affiliate=gem&affiliate_bps=50&streaming_interval=1&streaming_quantity=0"
        );
        assert_eq!(
            ThorChainTarget::TransactionStatus {
                network: THORChainNetwork::Mayachain,
                hash: "ABC".into()
            }
            .path(),
            "/mayachain/tx/status/ABC"
        );
    }
}
