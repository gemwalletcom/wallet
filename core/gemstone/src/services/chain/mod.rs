pub mod rules;

use primitives::{Chain, ChainAsset};

use crate::config::chain::icon_chain;
use crate::services::assets::icon::{GemAssetIcon, GemAssetIconImage};
use crate::services::localization::GemLocalizedText;

use crate::wallet_connect::{wallet_connect_namespace, wallet_connect_reference};

/// A network as a list row draws it: its name, the token standard when a screen names one, and its icon.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemChainRow {
    pub chain: Chain,
    pub title: String,
    pub standard: Option<GemLocalizedText>,
    pub icon: GemAssetIcon,
}

#[uniffi::export]
pub fn chain_row(chain: Chain) -> GemChainRow {
    chain_row_with_standard(chain, None)
}

pub fn chain_row_with_standard(chain: Chain, standard: Option<GemLocalizedText>) -> GemChainRow {
    GemChainRow {
        chain,
        title: ChainAsset::from_chain(chain).network_name,
        standard,
        icon: GemAssetIcon {
            image: GemAssetIconImage::Local { chain: icon_chain(chain) },
            badge: None,
            placeholder: None,
        },
    }
}

#[derive(Default, uniffi::Object)]
pub struct GemChainService {}

#[uniffi::export]
impl GemChainService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_chains(&self, query: String) -> Vec<Chain> {
        rules::matching_chains(rules::chains_by_rank(), &query)
    }

    pub fn get_matching_chains(&self, chains: Vec<Chain>, query: String) -> Vec<Chain> {
        rules::matching_chains(chains, &query)
    }

    pub fn caip2_namespace(&self, chain: Chain) -> Option<String> {
        wallet_connect_namespace(chain)
    }

    pub fn caip2_reference(&self, chain: Chain) -> Option<String> {
        wallet_connect_reference(chain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_chain_row_names_the_network_and_draws_its_icon_chain() {
        let row = chain_row(Chain::SeiEvm);

        assert_eq!(row.title, ChainAsset::from_chain(Chain::SeiEvm).network_name);
        assert_eq!(row.icon.image, GemAssetIconImage::Local { chain: icon_chain(Chain::SeiEvm) });
        assert_eq!((row.icon.badge, row.standard), (None, None));
    }
}
