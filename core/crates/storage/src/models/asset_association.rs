use diesel::prelude::*;
use primitives::AssetAssociation;
use serde::{Deserialize, Serialize};

use crate::sql_types::{AssetAssociationType as AssetAssociationTypeRow, AssetId};

#[derive(Debug, Queryable, Selectable, Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::assets_associations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssetAssociationRow {
    pub asset_id: AssetId,
    pub id: String,
    pub association_type: AssetAssociationTypeRow,
}

impl AssetAssociationRow {
    pub fn from_primitive(id: &str, association: AssetAssociation) -> Self {
        Self {
            asset_id: association.asset_id.into(),
            id: id.to_string(),
            association_type: association.association_type.into(),
        }
    }

    pub fn as_primitive(self) -> AssetAssociation {
        AssetAssociation {
            asset_id: self.asset_id.into(),
            association_type: self.association_type.0,
        }
    }
}
