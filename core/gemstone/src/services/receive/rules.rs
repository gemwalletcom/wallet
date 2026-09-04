use primitives::{AssetId, Chain, Wallet};

use super::model::GemMemoWarning;
use crate::config::chain::is_memo_supported;

pub fn memo_warning(chain: Chain) -> GemMemoWarning {
    if !is_memo_supported(chain) {
        return GemMemoWarning::NotSupported;
    }
    match chain {
        Chain::Xrp => GemMemoWarning::DestinationTag,
        _ => GemMemoWarning::Memo,
    }
}

pub fn network_asset_ids(asset_id: AssetId, associations: Vec<AssetId>, wallet: &Wallet) -> Vec<AssetId> {
    let mut asset_ids: Vec<AssetId> = Vec::new();
    for candidate in std::iter::once(asset_id).chain(associations) {
        if wallet.account(candidate.chain).is_some() && !asset_ids.contains(&candidate) {
            asset_ids.push(candidate);
        }
    }
    asset_ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Account;

    #[test]
    fn test_network_asset_ids_keeps_wallet_chains_in_order_without_duplicates() {
        let ethereum_usdc = AssetId::from_token(Chain::Ethereum, "0xusdc");
        let base_usdc = AssetId::from_token(Chain::Base, "0xusdc");
        let solana_usdc = AssetId::from_token(Chain::Solana, "usdc");

        let asset_ids = network_asset_ids(
            ethereum_usdc.clone(),
            vec![base_usdc.clone(), solana_usdc, ethereum_usdc.clone()],
            &Wallet::mock_with_accounts(Account::mock_chains(&[Chain::Ethereum, Chain::Base], "address")),
        );

        assert_eq!(asset_ids, vec![ethereum_usdc, base_usdc]);
    }

    #[test]
    fn test_memo_warning_names_the_field_each_chain_uses() {
        assert_eq!(memo_warning(Chain::Xrp), GemMemoWarning::DestinationTag);
        assert_eq!(memo_warning(Chain::Cosmos), GemMemoWarning::Memo);
        assert_eq!(memo_warning(Chain::Ton), GemMemoWarning::Memo);
        assert_eq!(memo_warning(Chain::Ethereum), GemMemoWarning::NotSupported);
        assert_eq!(memo_warning(Chain::Bitcoin), GemMemoWarning::NotSupported);
    }
}
