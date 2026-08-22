use std::collections::HashSet;
use std::error::Error;

use async_trait::async_trait;
use gem_tracing::info_with_fields;
use prices::{AssetPriceMapping, PriceProviders};
use primitives::{AssetAssociation, AssetAssociationType, AssetId};
use storage::{AssetsRepository, Database};
use streamer::{FetchAssetAssociationsPayload, consumer::MessageConsumer};

pub struct FetchAssetAssociationsConsumer {
    pub database: Database,
    pub providers: PriceProviders,
}

#[async_trait]
impl MessageConsumer<FetchAssetAssociationsPayload, usize> for FetchAssetAssociationsConsumer {
    async fn should_process(&self, _payload: &FetchAssetAssociationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: FetchAssetAssociationsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let price_id = &payload.price_id;
        let provider = self
            .providers
            .get(&price_id.provider)
            .ok_or_else(|| format!("Unsupported asset association price provider: {}", price_id.provider))?;
        let mappings = provider.get_mappings_for_price_id(&price_id.provider_price_id).await?;
        let discovered_asset_ids = mappings.iter().map(|mapping| mapping.asset_id.clone()).collect();
        let existing_asset_ids = self
            .database
            .assets()?
            .get_assets(discovered_asset_ids)?
            .into_iter()
            .map(|asset| asset.id)
            .collect::<HashSet<_>>();
        let associations = map_asset_associations(mappings, &existing_asset_ids);

        if associations.len() < 2 {
            return Err(format!("Price association has fewer than two existing assets: {price_id}").into());
        }

        let count = self.database.assets()?.upsert_asset_associations(&payload.id, associations)?;
        info_with_fields!("fetch asset associations", id = payload.id.as_str(), count = count);
        Ok(count)
    }
}

fn map_asset_associations(mappings: Vec<AssetPriceMapping>, existing_asset_ids: &HashSet<AssetId>) -> Vec<AssetAssociation> {
    let mut associations = mappings
        .into_iter()
        .filter(|mapping| existing_asset_ids.contains(&mapping.asset_id))
        .map(|mapping| AssetAssociation {
            asset_id: mapping.asset_id,
            association_type: AssetAssociationType::Official,
        })
        .collect::<Vec<_>>();
    associations.sort_by_key(|association| association.asset_id.to_string());
    associations
}

#[cfg(test)]
mod tests {
    use primitives::Chain;

    use super::*;

    #[test]
    fn test_map_asset_associations() {
        let ethereum = AssetId::from_token(Chain::Ethereum, "0xdC035D45d973E3EC169d2276DDab16f1e407384F");
        let base = AssetId::from_token(Chain::Base, "0x820C137fa70C8691f0e44Dc420a5e53c168921Dc");
        let solana = AssetId::from_token(Chain::Solana, "USDSwr9ApdHk5bvJKMjzff41FfuX8bSxdKcR81vTwcA");
        let mappings = vec![
            AssetPriceMapping::new(ethereum.clone(), "usds".to_string()),
            AssetPriceMapping::new(base, "usds".to_string()),
            AssetPriceMapping::new(solana.clone(), "usds".to_string()),
        ];
        let existing_asset_ids = HashSet::from([ethereum.clone(), solana.clone()]);

        assert_eq!(
            map_asset_associations(mappings, &existing_asset_ids),
            vec![
                AssetAssociation {
                    asset_id: ethereum,
                    association_type: AssetAssociationType::Official,
                },
                AssetAssociation {
                    asset_id: solana,
                    association_type: AssetAssociationType::Official,
                },
            ]
        );
    }
}
