use crate::DatabaseError;
use diesel::prelude::*;
use num_bigint::BigUint;
use primitives::{AssetAddress, AssetId as PrimitiveAssetId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;

use crate::sql_types::{AssetId, ChainRow};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::assets_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct AssetAddressRow {
    pub chain: ChainRow,
    pub asset_id: AssetId,
    pub address: String,
    pub value: Option<String>,
}

impl AssetAddressRow {
    pub fn from_primitive(asset_address: AssetAddress) -> Self {
        Self {
            chain: ChainRow::from(asset_address.asset_id.chain),
            asset_id: asset_address.asset_id.into(),
            address: asset_address.address.clone(),
            value: asset_address.value.as_ref().map(BigUint::to_string),
        }
    }

    pub fn as_primitive(&self) -> Result<AssetAddress, DatabaseError> {
        let value = self
            .value
            .as_deref()
            .map(|value| BigUint::from_str(value).map_err(|error| DatabaseError::Error(format!("Asset address {} has an invalid value: {error}", self.address))))
            .transpose()?;
        Ok(AssetAddress {
            asset_id: self.asset_id.0.clone(),
            address: self.address.clone(),
            value,
        })
    }
}

pub trait AssetAddressRowsExt {
    fn asset_ids(self) -> Vec<PrimitiveAssetId>;
}

impl AssetAddressRowsExt for Vec<AssetAddressRow> {
    fn asset_ids(self) -> Vec<PrimitiveAssetId> {
        let mut seen = HashSet::new();

        self.into_iter().filter_map(|row| seen.insert(row.asset_id.0.clone()).then_some(row.asset_id.0)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Asset, Chain, asset_constants::ETHEREUM_USDC_ASSET_ID};

    #[test]
    fn test_asset_ids() {
        let eth = Asset::from_chain(Chain::Ethereum).id;
        let usdc = ETHEREUM_USDC_ASSET_ID.clone();
        let row = |asset_id: &PrimitiveAssetId, address: &str| AssetAddressRow {
            chain: ChainRow::from(Chain::Ethereum),
            asset_id: asset_id.clone().into(),
            address: address.to_string(),
            value: None,
        };
        let rows = vec![row(&eth, "0xwallet"), row(&usdc, "0xwallet"), row(&usdc, "0xother")];

        assert_eq!(rows.asset_ids(), vec![eth, usdc]);
    }
}
