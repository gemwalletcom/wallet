use primitives::{
    Asset, AssetId, AssetProperties, AssetScore, AssetType, Chain, ChainAsset, WalletType,
    known_assets::{HYPERCORE_PERPETUAL_USDC, HYPERCORE_SPOT_USDC, SOLANA_USDC, SOLANA_USDT, TEMPO_PATHUSD, TEMPO_USDC, TRON_USDT},
};

pub type GemAsset = Asset;
pub type GemAssetType = AssetType;
pub type GemChainAsset = ChainAsset;

#[uniffi::remote(Record)]
pub struct GemChainAsset {
    pub asset: GemAsset,
    pub network_name: String,
}

#[allow(non_camel_case_types)]
#[uniffi::remote(Enum)]
pub enum GemAssetType {
    NATIVE,
    ERC20,
    BEP20,
    SPL,
    SPL2022,
    TRC20,
    TIP20,
    TOKEN,
    IBC,
    JETTON,
    SYNTH,
    ASA,
    PERPETUAL,
    SPOT,
}

#[uniffi::remote(Record)]
pub struct GemAsset {
    pub id: AssetId,
    pub chain: Chain,
    pub token_id: Option<String>,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub asset_type: GemAssetType,
}

#[uniffi::export]
pub fn asset_default_rank(chain: Chain) -> i32 {
    chain.rank()
}

#[uniffi::export]
pub fn default_token_rank() -> i32 {
    AssetScore::default().rank
}

#[uniffi::export]
pub fn wallet_default_assets(chain: Chain) -> Vec<GemAsset> {
    match chain {
        Chain::HyperCore => vec![HYPERCORE_PERPETUAL_USDC.clone(), HYPERCORE_SPOT_USDC.clone()],
        Chain::Solana => vec![SOLANA_USDC.clone(), SOLANA_USDT.clone()],
        Chain::Tempo => vec![TEMPO_USDC.clone(), TEMPO_PATHUSD.clone()],
        Chain::Tron => vec![TRON_USDT.clone()],
        _ => vec![],
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

    #[test]
    fn test_wallet_asset_is_enabled() {
        assert!(wallet_asset_is_enabled(TEMPO_USDC.id.clone(), WalletType::Single));
        assert!(wallet_asset_is_enabled(TEMPO_PATHUSD.id.clone(), WalletType::Single));
        assert!(!wallet_asset_is_enabled(TEMPO_USDC.id.clone(), WalletType::Multicoin));
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
}
