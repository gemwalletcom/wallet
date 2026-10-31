use super::constants::{FACTORY, FACTORY_DEPLOY_CODE, POOL_CONTRACT_TYPE, VAULT_CONTRACT_TYPE};
use crate::SwapperError;
use gem_ton::{
    address::Address,
    models::StackArg,
    tvm::{BagOfCells, Cell, CellArc, CellBuilder, TvmError},
};
use primitives::AssetId;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static DEPLOY_CODE: LazyLock<Result<CellArc, TvmError>> = LazyLock::new(|| BagOfCells::parse_base64_root(FACTORY_DEPLOY_CODE));

const NATIVE_ASSET_TAG: u8 = 0b0000;
const JETTON_ASSET_TAG: u8 = 0b0001;

fn factory_contract_address(contract: &Cell) -> Result<Address, SwapperError> {
    let code = DEPLOY_CODE.as_ref().map_err(Clone::clone)?;
    let factory = Address::parse(FACTORY)?;
    let data = CellBuilder::new().store_address(&factory)?.store_cell_data(contract)?.end_cell()?.into_arc();
    let state_init = CellBuilder::new().store_u8(2, 0)?.store_maybe_reference(Some(code))?.store_maybe_reference(Some(&data))?.store_bit(false)?.end_cell()?;
    Ok(Address::new(factory.workchain(), state_init.hash))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum DedustAsset {
    Native,
    Jetton(Address),
}

impl DedustAsset {
    pub fn from_asset_id(asset_id: &AssetId) -> Result<Self, SwapperError> {
        match &asset_id.token_id {
            None => Ok(Self::Native),
            Some(token_id) => Ok(Self::Jetton(Address::parse(token_id)?)),
        }
    }

    pub fn stack_arg(&self) -> Result<StackArg, TvmError> {
        Ok(StackArg::slice(self.cell()?.to_boc_base64()?))
    }

    pub fn vault_address(&self) -> Result<Address, SwapperError> {
        factory_contract_address(&CellBuilder::new().store_u8(8, VAULT_CONTRACT_TYPE)?.store_cell_data(&self.cell()?)?.end_cell()?)
    }

    fn sort_key(&self) -> Option<&[u8]> {
        match self {
            Self::Native => None,
            Self::Jetton(address) => Some(address.hash_part().as_slice()),
        }
    }

    fn cell(&self) -> Result<Cell, TvmError> {
        match self {
            Self::Native => CellBuilder::new().store_u8(4, NATIVE_ASSET_TAG)?.end_cell(),
            Self::Jetton(address) => CellBuilder::new().store_u8(4, JETTON_ASSET_TAG)?.store_i32(8, address.workchain())?.store_slice(address.hash_part())?.end_cell(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum PoolType {
    Volatile,
    Stable,
}

impl PoolType {
    pub const ALL: [Self; 2] = [Self::Volatile, Self::Stable];

    pub fn pool_address(self, left: &DedustAsset, right: &DedustAsset) -> Result<Address, SwapperError> {
        let (first, second) = if left.sort_key() <= right.sort_key() { (left, right) } else { (right, left) };
        factory_contract_address(
            &CellBuilder::new()
                .store_u8(8, POOL_CONTRACT_TYPE)?
                .store_bit(self == Self::Stable)?
                .store_cell_data(&first.cell()?)?
                .store_cell_data(&second.cell()?)?
                .end_cell()?,
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct PoolRoute {
    pub pools: Vec<String>,
}

impl PoolRoute {
    pub fn new(pools: &[Address]) -> Self {
        Self {
            pools: pools.iter().map(Address::encode_bounceable).collect(),
        }
    }

    pub fn pool_addresses(&self) -> Result<Vec<Address>, SwapperError> {
        self.pools.iter().map(|pool| Ok(Address::parse(pool)?)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dedust::testkit::{NATIVE_VAULT, NOT_TOKEN_ID, STTON_TOKEN_ID, TON_NOT_POOL, TON_STTON_STABLE_POOL, TON_USDT_POOL, USDT_NOT_POOL, USDT_VAULT};
    use primitives::asset_constants::TON_USDT_TOKEN_ID;

    #[test]
    fn test_vault_address() {
        let vault = |asset: DedustAsset| asset.vault_address().unwrap().encode_bounceable();
        let jetton = |token_id: &str| DedustAsset::Jetton(Address::parse(token_id).unwrap());

        assert_eq!(vault(DedustAsset::Native), NATIVE_VAULT);
        assert_eq!(vault(jetton(TON_USDT_TOKEN_ID)), USDT_VAULT);
        assert_eq!(vault(jetton(NOT_TOKEN_ID)), "EQAJxbmwhwJw-FomFlt1RYukdeTltiCmWlsurQsQMRJhZfdB");
        assert_eq!(vault(jetton(STTON_TOKEN_ID)), "EQACpR7Dc3393EVHkZ-7pg7zZMB5j7DAh2NNteRzK2wPGqk1");
    }

    #[test]
    fn test_pool_address() {
        let jetton = |token_id: &str| DedustAsset::Jetton(Address::parse(token_id).unwrap());
        let usdt = jetton(TON_USDT_TOKEN_ID);
        let not = jetton(NOT_TOKEN_ID);
        let pool = |pool_type: PoolType, left: &DedustAsset, right: &DedustAsset| pool_type.pool_address(left, right).unwrap().encode_bounceable();

        assert_eq!(pool(PoolType::Volatile, &DedustAsset::Native, &usdt), TON_USDT_POOL);
        assert_eq!(pool(PoolType::Volatile, &usdt, &DedustAsset::Native), TON_USDT_POOL);
        assert_eq!(pool(PoolType::Volatile, &DedustAsset::Native, &not), TON_NOT_POOL);
        assert_eq!(pool(PoolType::Stable, &DedustAsset::Native, &jetton(STTON_TOKEN_ID)), TON_STTON_STABLE_POOL);
        assert_eq!(pool(PoolType::Volatile, &usdt, &not), USDT_NOT_POOL);
        assert_eq!(pool(PoolType::Volatile, &not, &usdt), USDT_NOT_POOL);
    }
}
