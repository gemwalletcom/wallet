use crate::SwapperError;
use gem_ton::{
    address::Address,
    models::StackArg,
    tvm::{BagOfCells, Cell, CellBuilder, TvmError},
};
use primitives::AssetId;
use serde::{Deserialize, Serialize};

const NATIVE_ASSET_TAG: u8 = 0b0000;
const JETTON_ASSET_TAG: u8 = 0b0001;

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
        Ok(StackArg::slice(BagOfCells::from_root(self.cell()?).to_base64(true)?))
    }

    fn cell(&self) -> Result<Cell, TvmError> {
        match self {
            Self::Native => CellBuilder::new().store_u8(4, NATIVE_ASSET_TAG)?.end_cell(),
            Self::Jetton(address) => CellBuilder::new().store_u8(4, JETTON_ASSET_TAG)?.store_i32(8, address.workchain())?.store_slice(address.hash_part())?.end_cell(),
        }
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
