use diesel::prelude::*;
use primitives::{AssetId, AssetTag, ListId, PerpetualId, TagVisibility};

use crate::models::{AssetTagRow, NewListTagRow, PerpetualTagRow, TagRow};
use crate::{DatabaseClient, DatabaseError};

#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub visibility: TagVisibility,
    pub list_id: Option<ListId>,
}

impl Tag {
    pub fn is_public(&self) -> bool {
        self.visibility == TagVisibility::Public
    }

    fn from_row(row: TagRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            visibility: row.visibility.0,
            list_id: row.list_id.map(|list_id| list_id.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetTagLink {
    pub asset_id: AssetId,
    pub tag_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerpetualTagLink {
    pub perpetual_id: PerpetualId,
    pub tag_id: String,
}

pub trait TagRepository {
    fn add_tags(&mut self, values: Vec<AssetTag>) -> Result<usize, DatabaseError>;
    fn add_list_tag(&mut self, tag_id: &str, name: &str, list_id: ListId) -> Result<usize, DatabaseError>;
    fn get_tag(&mut self, tag_id: &str) -> Result<Option<Tag>, DatabaseError>;
    fn get_list_tags(&mut self) -> Result<Vec<Tag>, DatabaseError>;
    fn get_asset_list_tags(&mut self) -> Result<Vec<Tag>, DatabaseError>;
    fn get_perpetual_list_tags(&mut self) -> Result<Vec<Tag>, DatabaseError>;
    fn get_assets_tags(&mut self) -> Result<Vec<AssetTagLink>, DatabaseError>;
    fn get_perpetuals_tags(&mut self) -> Result<Vec<PerpetualTagLink>, DatabaseError>;
    fn get_asset_ids_for_tag(&mut self, tag_id: &str) -> Result<Vec<AssetId>, DatabaseError>;
    fn set_assets_tags_for_tag(&mut self, tag_id: &str, asset_ids: Vec<AssetId>) -> Result<usize, DatabaseError>;
}

pub(crate) fn asset_tag_ids(client: &mut DatabaseClient, value: &AssetId) -> Result<Vec<String>, diesel::result::Error> {
    use crate::schema::assets_tags::dsl::*;
    assets_tags.filter(asset_id.eq(value.to_string())).select(tag_id).load(&mut client.connection)
}

fn tags_from_rows(rows: Vec<TagRow>) -> Vec<Tag> {
    rows.into_iter().map(Tag::from_row).collect()
}

impl TagRepository for DatabaseClient {
    fn add_tags(&mut self, values: Vec<AssetTag>) -> Result<usize, DatabaseError> {
        use crate::schema::tags::dsl::*;
        let rows = values.into_iter().map(TagRow::from_primitive).collect::<Vec<_>>();
        Ok(diesel::insert_into(tags).values(rows).on_conflict_do_nothing().execute(&mut self.connection)?)
    }

    fn add_list_tag(&mut self, tag_id: &str, name: &str, list_id: ListId) -> Result<usize, DatabaseError> {
        use crate::schema::tags::dsl::tags;
        Ok(diesel::insert_into(tags).values(NewListTagRow::new(tag_id, name, list_id)).on_conflict_do_nothing().execute(&mut self.connection)?)
    }

    fn get_tag(&mut self, tag_id: &str) -> Result<Option<Tag>, DatabaseError> {
        use crate::schema::tags::dsl::tags;
        Ok(tags.find(tag_id).select(TagRow::as_select()).first(&mut self.connection).optional()?.map(Tag::from_row))
    }

    fn get_list_tags(&mut self) -> Result<Vec<Tag>, DatabaseError> {
        use crate::schema::tags::dsl::*;
        Ok(tags_from_rows(tags.filter(list_id.is_not_null()).order(id.asc()).select(TagRow::as_select()).load(&mut self.connection)?))
    }

    fn get_asset_list_tags(&mut self) -> Result<Vec<Tag>, DatabaseError> {
        use crate::schema::{assets_tags, tags};
        Ok(tags_from_rows(
            tags::table.inner_join(assets_tags::table).select(TagRow::as_select()).distinct().order(tags::id.asc()).load(&mut self.connection)?,
        ))
    }

    fn get_perpetual_list_tags(&mut self) -> Result<Vec<Tag>, DatabaseError> {
        use crate::schema::{perpetuals_tags, tags};
        Ok(tags_from_rows(
            tags::table.inner_join(perpetuals_tags::table).select(TagRow::as_select()).distinct().order(tags::id.asc()).load(&mut self.connection)?,
        ))
    }

    fn get_assets_tags(&mut self) -> Result<Vec<AssetTagLink>, DatabaseError> {
        use crate::schema::assets_tags::dsl::*;
        let rows = assets_tags.select(AssetTagRow::as_select()).load(&mut self.connection)?;
        Ok(rows
            .into_iter()
            .map(|row| AssetTagLink {
                asset_id: row.asset_id.0,
                tag_id: row.tag_id,
            })
            .collect())
    }

    fn get_perpetuals_tags(&mut self) -> Result<Vec<PerpetualTagLink>, DatabaseError> {
        use crate::schema::perpetuals_tags::dsl::*;
        let rows = perpetuals_tags.select(PerpetualTagRow::as_select()).load(&mut self.connection)?;
        Ok(rows
            .into_iter()
            .map(|row| PerpetualTagLink {
                perpetual_id: row.perpetual_id.0,
                tag_id: row.tag_id,
            })
            .collect())
    }

    fn get_asset_ids_for_tag(&mut self, value: &str) -> Result<Vec<AssetId>, DatabaseError> {
        use crate::schema::assets_tags::dsl::*;
        let rows = assets_tags.filter(tag_id.eq(value)).order(order.asc()).select(AssetTagRow::as_select()).load(&mut self.connection)?;
        Ok(rows.into_iter().map(|row| row.asset_id.0).collect())
    }

    fn set_assets_tags_for_tag(&mut self, value: &str, asset_ids: Vec<AssetId>) -> Result<usize, DatabaseError> {
        use crate::schema::assets_tags::dsl::*;
        let values = asset_ids
            .into_iter()
            .enumerate()
            .map(|(index, current_asset_id)| AssetTagRow {
                asset_id: current_asset_id.into(),
                tag_id: value.to_string(),
                order: Some(index as i32),
            })
            .collect::<Vec<_>>();

        Ok(self.connection.transaction::<_, diesel::result::Error, _>(|conn| {
            let deleted_count = diesel::delete(assets_tags.filter(tag_id.eq(value))).execute(conn)?;
            if values.is_empty() { Ok(deleted_count) } else { diesel::insert_into(assets_tags).values(values).execute(conn) }
        })?)
    }
}

#[cfg(all(test, feature = "database_integration_tests"))]
mod database_integration_tests {
    use primitives::{Asset, AssetId, Chain, ListId, ListProviderName, TagVisibility};

    use crate::{AssetTagLink, AssetsRepository, ChainsRepository, Database, DatabaseError, TagRepository};

    const TAG_ID: &str = "test-list-tag";

    #[tokio::test]
    async fn test_list_tag_with_assets() {
        let database = Database::mock();
        let list_id = ListId {
            provider: ListProviderName::Coingecko,
            provider_list_id: "test-category".to_string(),
        };
        let asset_ids = vec![AssetId::from_chain(Chain::Bitcoin), AssetId::from_chain(Chain::Ethereum)];
        let expected_list_id = list_id.clone();
        let tagged = asset_ids.clone();
        let (tag, list_tags, asset_list_tags, tag_asset_ids, links) = database
            .run(move |client| -> Result<_, DatabaseError> {
                client.add_chains(vec![Chain::Bitcoin, Chain::Ethereum])?;
                client.add_assets(vec![Asset::from_chain(Chain::Bitcoin).as_basic_primitive(), Asset::from_chain(Chain::Ethereum).as_basic_primitive()])?;
                client.add_list_tag(TAG_ID, "Test List", list_id)?;
                client.set_assets_tags_for_tag(TAG_ID, tagged)?;
                Ok((
                    client.get_tag(TAG_ID)?,
                    client.get_list_tags()?,
                    client.get_asset_list_tags()?,
                    client.get_asset_ids_for_tag(TAG_ID)?,
                    client.get_assets_tags()?,
                ))
            })
            .await
            .unwrap();

        let tag = tag.unwrap();
        assert_eq!(tag.name, "Test List");
        assert_eq!(tag.visibility, TagVisibility::Public);
        assert_eq!(tag.list_id, Some(expected_list_id));
        assert!(list_tags.contains(&tag));
        assert!(asset_list_tags.contains(&tag));
        assert_eq!(tag_asset_ids, asset_ids);
        assert!(links.contains(&AssetTagLink {
            asset_id: AssetId::from_chain(Chain::Ethereum),
            tag_id: TAG_ID.to_string(),
        }));
    }
}
