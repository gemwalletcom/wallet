use diesel::prelude::*;

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::nft_assets_associations)]
pub struct NewNftAssetAssociationRow {
    pub address_id: i32,
    pub asset_id: i32,
}
