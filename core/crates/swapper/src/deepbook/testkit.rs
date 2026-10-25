use super::model::Pool;
use gem_sui::SUI_COIN_TYPE_FULL;
use primitives::asset_constants::SUI_USDC_TOKEN_ID;

impl Pool {
    pub fn mock() -> Self {
        Self {
            pool_id: "0xe05dafb5133bcffb8d59f4e12465dc0e9faeaa05e3e342a08fe135800e3e4407".into(),
            initial_shared_version: 389_750_322,
            base_coin_type: SUI_COIN_TYPE_FULL.into(),
            quote_coin_type: SUI_USDC_TOKEN_ID.into(),
        }
    }
}
