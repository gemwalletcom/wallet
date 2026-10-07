use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::{AssetLink, NFTCollection, NFTImages, NFTResource, VerificationStatus};
use serde::{Deserialize, Serialize};

use crate::sql_types::{ChainRow, NftCollectionIdRow};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::nft_collections)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct NftCollectionRow {
    pub id: i32,
    pub identifier: NftCollectionIdRow,
    pub chain: ChainRow,
    pub name: String,
    pub description: String,
    pub symbol: Option<String>,
    pub contract_address: String,
    pub image_preview_url: String,
    pub image_preview_mime_type: String,
    pub is_verified: bool,
    pub is_enabled: bool,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::nft_collections)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct NewNftCollectionRow {
    pub identifier: NftCollectionIdRow,
    pub chain: ChainRow,
    pub name: String,
    pub description: String,
    pub symbol: Option<String>,
    pub contract_address: String,
    pub image_preview_url: String,
    pub image_preview_mime_type: String,
    pub is_verified: bool,
    pub is_enabled: bool,
}

impl NewNftCollectionRow {
    pub fn from_primitive(collection: NFTCollection) -> Self {
        NewNftCollectionRow {
            identifier: collection.id.clone().into(),
            name: collection.name.clone(),
            description: collection.description.unwrap_or_default(),
            chain: ChainRow::from(collection.chain),
            image_preview_url: collection.images.preview.url.clone(),
            image_preview_mime_type: collection.images.preview.mime_type.clone(),
            is_verified: collection.status.is_verified(),
            symbol: collection.symbol,
            contract_address: collection.contract_address,
            is_enabled: true,
        }
    }
}

impl NftCollectionRow {
    pub fn as_primitive(&self, links: Vec<AssetLink>) -> NFTCollection {
        NFTCollection {
            id: self.identifier.0.clone(),
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            description: Some(self.description.clone()),
            chain: self.chain.0,
            contract_address: self.contract_address.clone(),
            images: NFTImages {
                preview: NFTResource {
                    url: self.image_preview_url.clone(),
                    mime_type: self.image_preview_mime_type.clone(),
                },
            },
            status: VerificationStatus::from_verified(self.is_verified),
            links,
        }
    }
}
