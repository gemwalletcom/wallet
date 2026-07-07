use diesel::prelude::*;
use primitives::{AssetList, AssetTag, ListId};
use serde::{Deserialize, Serialize};

use crate::sql_types::{AssetId, ListIdRow, PerpetualIdRow, TagVisibility};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TagRow {
    pub id: String,
    pub name: String,
    pub visibility: TagVisibility,
    pub list_id: Option<ListIdRow>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewListTagRow {
    pub id: String,
    pub name: String,
    pub list_id: ListIdRow,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::assets_tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssetTagRow {
    pub asset_id: AssetId,
    pub tag_id: String,
    pub order: Option<i32>,
}

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::perpetuals_tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PerpetualTagRow {
    pub perpetual_id: PerpetualIdRow,
    pub tag_id: String,
    pub order: Option<i32>,
}

impl TagRow {
    pub fn from_primitive(primitive: AssetTag) -> Self {
        let id = primitive.as_ref().to_lowercase();
        let name = id.clone();
        Self {
            id,
            name,
            visibility: TagVisibility::Public,
            list_id: None,
        }
    }

    pub fn as_primitive(&self, count: u32) -> AssetList {
        AssetList {
            id: self.id.clone(),
            name: self.name.clone(),
            count,
        }
    }
}

impl NewListTagRow {
    pub fn new(id: &str, name: &str, list_id: ListId) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            list_id: ListIdRow::from(list_id),
        }
    }
}
