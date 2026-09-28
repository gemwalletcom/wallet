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

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemImportWalletTypes {
    pub multicoin: GemLocalizedText,
    pub chains: Vec<GemChainRow>,
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

    /// Every chain by rank, or only `chains` in their order, that matches `query`, as the rows each chain picker draws.
    pub fn chain_rows(&self, chains: Option<Vec<Chain>>, query: String) -> Vec<GemChainRow> {
        rules::matching_chains(chains.unwrap_or_else(rules::chains_by_rank), &query).into_iter().map(chain_row).collect()
    }

    pub fn import_wallet_types(&self, query: String) -> GemImportWalletTypes {
        GemImportWalletTypes {
            multicoin: GemLocalizedText::WalletMulticoin,
            chains: self.chain_rows(None, query),
        }
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
    fn test_the_import_types_lead_with_multicoin_and_list_the_matching_chains() {
        let types = GemChainService::new().import_wallet_types("bitcoin".to_string());

        assert_eq!(types.multicoin, GemLocalizedText::WalletMulticoin);
        assert_eq!(types.chains.first().map(|row| row.chain), Some(Chain::Bitcoin));
        assert!(GemChainService::new().import_wallet_types("zzz-no-chain".to_string()).chains.is_empty());
    }

    #[test]
    fn test_chain_rows_search_every_chain_or_only_the_ones_offered() {
        let service = GemChainService::new();
        let all = service.chain_rows(None, String::new());
        let offered = service.chain_rows(Some(vec![Chain::Solana, Chain::Bitcoin]), String::new());

        assert_eq!(all.len(), rules::chains_by_rank().len());
        assert_eq!(offered.iter().map(|row| row.chain).collect::<Vec<_>>(), vec![Chain::Solana, Chain::Bitcoin], "offered chains keep their order");
        assert_eq!(service.chain_rows(Some(vec![Chain::Solana, Chain::Bitcoin]), "bitcoin".to_string()), vec![chain_row(Chain::Bitcoin)]);
        assert!(service.chain_rows(None, "zzz-no-chain".to_string()).is_empty());
    }

    #[test]
    fn test_a_chain_row_names_the_network_and_draws_its_icon_chain() {
        let row = chain_row(Chain::SeiEvm);

        assert_eq!(row.title, ChainAsset::from_chain(Chain::SeiEvm).network_name);
        assert_eq!(row.icon.image, GemAssetIconImage::Local { chain: icon_chain(Chain::SeiEvm) });
        assert_eq!((row.icon.badge, row.standard), (None, None));
    }
}
