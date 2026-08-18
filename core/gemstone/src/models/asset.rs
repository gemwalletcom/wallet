use primitives::{Asset, AssetId, AssetScore, AssetType, Chain, ChainAsset};

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
pub fn chain_asset_wrapper(chain: Chain) -> GemChainAsset {
    ChainAsset::from_chain(chain)
}
