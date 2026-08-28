use primitives::{Asset, AssetId, AssetProperties, AssetScore, AssetType, Chain, ChainAsset, WalletType, known_assets};

pub type GemAsset = Asset;
pub type GemAssetType = AssetType;
pub type GemChainAsset = ChainAsset;

#[uniffi::export]
pub fn asset_default_rank(asset_id: AssetId) -> i32 {
    asset_id.default_rank()
}

#[uniffi::export]
pub fn default_token_rank() -> i32 {
    AssetScore::default().rank
}

#[uniffi::export]
pub fn wallet_default_assets(chain: Chain) -> Vec<GemAsset> {
    known_assets::wallet_default_assets(chain)
}

#[uniffi::export]
pub fn chain_fee_asset_ids(chain: Chain) -> Vec<AssetId> {
    match chain {
        Chain::Tempo => wallet_default_assets(chain).into_iter().map(|asset| asset.id).collect(),
        _ => Vec::new(),
    }
}

#[uniffi::export]
pub fn asset_ids_enabled_by_default() -> Vec<AssetId> {
    [Chain::Bitcoin, Chain::Ethereum, Chain::SmartChain, Chain::Solana, Chain::Tron]
        .into_iter()
        .map(AssetId::from_chain)
        .chain(wallet_default_assets(Chain::Tron).into_iter().map(|asset| asset.id))
        .collect()
}

#[uniffi::export]
pub fn wallet_asset_is_enabled(asset_id: AssetId, wallet_type: WalletType) -> bool {
    match wallet_type {
        WalletType::Multicoin => asset_ids_enabled_by_default().contains(&asset_id),
        WalletType::Single | WalletType::PrivateKey | WalletType::View => {
            let is_native = asset_id.is_native() && asset_id.chain.rank() >= 0;
            let is_default = wallet_default_assets(asset_id.chain).iter().any(|asset| asset.id == asset_id);
            is_native || is_default
        }
    }
}

#[uniffi::export]
pub fn asset_is_swapable(asset_id: AssetId) -> bool {
    AssetProperties::default(asset_id).is_swapable
}

#[uniffi::export]
pub fn chain_asset_wrapper(chain: Chain) -> GemChainAsset {
    ChainAsset::from_chain(chain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::known_assets::{SOLANA_USDC, SOLANA_USDT, TEMPO_BRIDGED_USDC, TEMPO_PATHUSD, TEMPO_USDT0, TRON_USDT};

    #[test]
    fn test_wallet_asset_is_enabled() {
        assert!(wallet_asset_is_enabled(TEMPO_BRIDGED_USDC.id.clone(), WalletType::Single));
        assert!(wallet_asset_is_enabled(TEMPO_PATHUSD.id.clone(), WalletType::Single));
        assert!(wallet_asset_is_enabled(TEMPO_USDT0.id.clone(), WalletType::Single));
        assert!(!wallet_asset_is_enabled(TEMPO_BRIDGED_USDC.id.clone(), WalletType::Multicoin));
        assert!(!wallet_asset_is_enabled(AssetId::from_chain(Chain::Tempo), WalletType::Single));
        assert!(!wallet_asset_is_enabled(
            AssetId::from_token(Chain::Tempo, "0x20C0000000000000000000000000000000000001"),
            WalletType::Single,
        ));
        assert!(wallet_asset_is_enabled(SOLANA_USDC.id.clone(), WalletType::Single));
        assert!(wallet_asset_is_enabled(SOLANA_USDT.id.clone(), WalletType::Single));
        assert!(!wallet_asset_is_enabled(SOLANA_USDC.id.clone(), WalletType::Multicoin));
        assert!(wallet_asset_is_enabled(AssetId::from_chain(Chain::Solana), WalletType::Multicoin));
        assert!(wallet_asset_is_enabled(TRON_USDT.id.clone(), WalletType::Multicoin));
    }

    #[test]
    fn test_only_tempo_lets_the_fee_asset_be_chosen() {
        let fee_asset_ids = chain_fee_asset_ids(Chain::Tempo);

        assert!(!fee_asset_ids.is_empty());
        assert_eq!(fee_asset_ids, wallet_default_assets(Chain::Tempo).into_iter().map(|asset| asset.id).collect::<Vec<_>>());
        assert!(chain_fee_asset_ids(Chain::Ethereum).is_empty());
    }
}
