use diesel::prelude::*;
use primitives::{AssetAssociation, AssetId as PrimitiveAssetId};
use serde::{Deserialize, Serialize};

use crate::sql_types::{AssetAssociationType as AssetAssociationTypeRow, AssetId};

#[derive(Debug, Queryable, Selectable, Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::assets_associations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssetAssociationRow {
    pub asset_id: AssetId,
    pub associated_asset_id: AssetId,
    pub association_type: AssetAssociationTypeRow,
}

impl AssetAssociationRow {
    pub fn from_primitive(asset_id: &PrimitiveAssetId, association: AssetAssociation) -> Self {
        Self {
            asset_id: asset_id.into(),
            associated_asset_id: association.asset_id.into(),
            association_type: association.association_type.into(),
        }
    }

    pub fn as_primitive(self) -> AssetAssociation {
        AssetAssociation {
            asset_id: self.associated_asset_id.into(),
            association_type: self.association_type.0,
        }
    }
}
