use chrono::{DateTime, Utc};
use primitives::{AddressFormatStyle, Asset, BlockExplorerLink, Chain, NFTAssetData, NFTAttribute, NFTAttributeType, NFTData, VerificationStatus, WalletType};

use super::model::{GemCollectibleAction, GemCollectibleAttribute, GemCollectibleAttributeValue, GemCollectibleDetails, GemCollectibleSection, GemNftItem, GemNftList, GemNftListScreen, GemNftRow, GemNftUnverifiedRow};
use crate::address_formatter::format_address;
use crate::config::chain::supports_nft_transfer;
use crate::config::social::social_links;
use crate::models::copy::{GemCopy, GemCopyKind, address_copy};
use crate::models::list::{GemListRow, GemListRowTitle};
use crate::services::assets::rules::asset_text;
use crate::services::localization::GemLocalizedText;

const TOKEN_ID_ADDRESS_LENGTH: usize = 16;

fn unverified_collections(data: Vec<NFTData>) -> Vec<NFTData> {
    collections(data, false)
}

pub fn list_screen(data: &[NFTData], list: GemNftList) -> GemNftListScreen {
    GemNftListScreen {
        title: match list {
            GemNftList::Collection => match data.first().map(|item| item.collection.name.clone()) {
                Some(name) => GemLocalizedText::Text { text: name },
                None => GemLocalizedText::NftCollections,
            },
            GemNftList::Unverified => GemLocalizedText::NftUnverified,
            GemNftList::Collections | GemNftList::Avatar => GemLocalizedText::NftCollections,
        },
        offers_receive: !matches!(list, GemNftList::Unverified),
        syncs_on_appear: matches!(list, GemNftList::Collections | GemNftList::Avatar),
    }
}

pub fn unverified_row(data: Vec<NFTData>, list: GemNftList) -> Option<GemNftUnverifiedRow> {
    match list {
        GemNftList::Collections => {
            let count = unverified_collections(data).len();
            (count > 0).then(|| GemNftUnverifiedRow { count_text: count.to_string() })
        }
        GemNftList::Unverified | GemNftList::Collection | GemNftList::Avatar => None,
    }
}

pub fn list_items(data: Vec<NFTData>, list: GemNftList) -> Vec<GemNftItem> {
    let collections = match list {
        GemNftList::Collections | GemNftList::Avatar => collections(data, true),
        GemNftList::Unverified => collections(data, false),
        GemNftList::Collection => data.into_iter().filter(|item| !item.assets.is_empty()).collect(),
    };
    sorted_collections(collections)
        .into_iter()
        .flat_map(|data| match list {
            GemNftList::Collections | GemNftList::Unverified => vec![item(data)],
            GemNftList::Collection | GemNftList::Avatar => asset_items(data),
        })
        .collect()
}

pub fn search_collections(data: Vec<NFTData>, query: &str) -> Vec<GemNftItem> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    sorted_collections(data)
        .into_iter()
        .flat_map(|mut data| {
            if data.collection.name.to_lowercase().contains(&query) {
                return vec![item(data)];
            }
            data.assets.retain(|asset| asset.name.to_lowercase().contains(&query));
            data.assets.sort_by_key(|asset| asset.name.to_lowercase());
            asset_items(data)
        })
        .collect()
}

pub fn row(item: &GemNftItem) -> GemNftRow {
    match item {
        GemNftItem::Collection { data } => GemNftRow {
            id: data.collection.id.to_string(),
            title: data.collection.name.clone(),
            image_url: data.collection.images.preview.url.clone(),
            count_text: Some(data.assets.len().to_string()),
            is_verified: data.collection.status == VerificationStatus::Verified,
        },
        GemNftItem::Asset { data } => GemNftRow {
            id: data.asset.id.to_string(),
            title: data.asset.name.clone(),
            image_url: data.asset.images.preview.url.clone(),
            count_text: None,
            is_verified: data.collection.status == VerificationStatus::Verified,
        },
    }
}

fn item(mut data: NFTData) -> GemNftItem {
    if data.assets.len() == 1 {
        let asset = data.assets.remove(0);
        return GemNftItem::Asset {
            data: NFTAssetData { collection: data.collection, asset },
        };
    }
    GemNftItem::Collection { data }
}

fn asset_items(data: NFTData) -> Vec<GemNftItem> {
    data.assets
        .into_iter()
        .map(|asset| GemNftItem::Asset {
            data: NFTAssetData { collection: data.collection.clone(), asset },
        })
        .collect()
}

fn sorted_collections(data: Vec<NFTData>) -> Vec<NFTData> {
    let mut sorted = data;
    sorted.sort_by(|left, right| right.assets.len().cmp(&left.assets.len()).then_with(|| left.collection.name.to_lowercase().cmp(&right.collection.name.to_lowercase())));
    sorted
}

pub fn nft_chains() -> Vec<Chain> {
    Chain::all().into_iter().filter(Chain::is_nft_supported).collect()
}

pub fn can_send(wallet_type: &WalletType, chain: Chain, is_owned: bool) -> bool {
    *wallet_type != WalletType::View && supports_nft_transfer(chain) && is_owned
}

pub fn collectible_details(wallet_type: &WalletType, data: &NFTAssetData, is_owned: bool, contract_explorer: Option<BlockExplorerLink>, token_explorer: Option<BlockExplorerLink>, can_save_image: bool) -> GemCollectibleDetails {
    let status = (data.collection.status != VerificationStatus::Verified).then_some(GemCollectibleSection::Status { status: data.collection.status });
    let info = GemCollectibleSection::Info {
        rows: info_rows(data, contract_explorer, token_explorer),
    };
    let attributes = (!data.asset.attributes.is_empty()).then(|| GemCollectibleSection::Attributes {
        attributes: data.asset.attributes.iter().map(attribute).collect(),
    });
    let links = Some(social_links(data.collection.links.clone())).filter(|links| !links.is_empty()).map(|links| GemCollectibleSection::Links { links });
    GemCollectibleDetails {
        can_send: can_send(wallet_type, data.asset.chain, is_owned),
        actions: collectible_actions(can_save_image),
        sections: [status, Some(info), attributes, links].into_iter().flatten().collect(),
    }
}

fn collectible_actions(can_save_image: bool) -> Vec<GemCollectibleAction> {
    [
        can_save_image.then_some(GemCollectibleAction::SaveImage),
        Some(GemCollectibleAction::SetAvatar),
        Some(GemCollectibleAction::Refresh),
        Some(GemCollectibleAction::Report),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn info_rows(data: &NFTAssetData, contract_explorer: Option<BlockExplorerLink>, token_explorer: Option<BlockExplorerLink>) -> Vec<GemListRow> {
    let chain = data.asset.chain;
    let token_id = &data.asset.token_id;
    let contract = &data.collection.contract_address;
    let contract_row = (!contract.is_empty() && contract != token_id).then(|| GemListRow::Identifier {
        title: GemListRowTitle::Contract,
        copy: address_copy(chain, contract.clone()),
        explorer: contract_explorer,
    });
    let token_text = if token_id.chars().count() > TOKEN_ID_ADDRESS_LENGTH {
        format_address(token_id, Some(chain), AddressFormatStyle::Short)
    } else {
        format!("#{token_id}")
    };
    [
        Some(GemListRow::Text {
            title: GemListRowTitle::Collection,
            value: data.collection.name.clone(),
        }),
        Some(GemListRow::Network {
            title: GemListRowTitle::Network,
            chain,
            name: asset_text(&Asset::from_chain(chain)).network_name,
        }),
        contract_row,
        Some(GemListRow::Identifier {
            title: GemListRowTitle::TokenId,
            copy: GemCopy {
                kind: GemCopyKind::Plain,
                value: token_id.clone(),
                display: token_text,
            },
            explorer: token_explorer,
        }),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn attribute(attribute: &NFTAttribute) -> GemCollectibleAttribute {
    let date = match attribute.value_type {
        Some(NFTAttributeType::Timestamp) => attribute_date(&attribute.value),
        Some(NFTAttributeType::String) | None => None,
    };
    GemCollectibleAttribute {
        name: attribute.name.clone(),
        value: date
            .map(|date| GemCollectibleAttributeValue::Date { date })
            .unwrap_or_else(|| GemCollectibleAttributeValue::Text { value: attribute.value.clone() }),
    }
}

fn attribute_date(value: &str) -> Option<DateTime<Utc>> {
    let value = value.trim();
    match value.parse::<i64>() {
        Ok(seconds) => DateTime::from_timestamp(seconds, 0),
        Err(_) => DateTime::parse_from_rfc3339(value).ok().map(|date| date.with_timezone(&Utc)),
    }
}

fn collections(data: Vec<NFTData>, verified: bool) -> Vec<NFTData> {
    data.into_iter().filter(|item| !item.assets.is_empty() && (item.collection.status == VerificationStatus::Verified) == verified).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetLink, LinkType, NFTAsset, NFTCollection, NFTData};

    #[test]
    fn test_a_collection_is_titled_by_its_name_and_only_unverified_offers_no_receive() {
        let data = vec![NFTData::mock()];
        let name = data[0].collection.name.clone();

        assert_eq!(list_screen(&data, GemNftList::Collection).title, GemLocalizedText::Text { text: name });
        assert_eq!(list_screen(&[], GemNftList::Collection).title, GemLocalizedText::NftCollections, "a collection with nothing in it still needs a title");
        assert_eq!(list_screen(&data, GemNftList::Collections).title, GemLocalizedText::NftCollections);
        assert_eq!(list_screen(&data, GemNftList::Unverified).title, GemLocalizedText::NftUnverified);

        assert!(list_screen(&data, GemNftList::Collections).offers_receive);
        assert!(list_screen(&data, GemNftList::Collection).offers_receive);
        assert!(!list_screen(&data, GemNftList::Unverified).offers_receive, "nothing unverified is worth asking for");

        assert!(list_screen(&data, GemNftList::Collections).syncs_on_appear);
        assert!(!list_screen(&data, GemNftList::Collection).syncs_on_appear, "a single collection is already in what the root synced; a pull still refetches");
        assert!(!list_screen(&data, GemNftList::Unverified).syncs_on_appear);
    }

    #[test]
    fn test_a_collection_row_counts_its_assets_and_an_asset_row_does_not() {
        let data = NFTData::mock_with("zebra", VerificationStatus::Verified, 2);
        let collection = row(&GemNftItem::Collection { data: data.clone() });
        let asset = row(&GemNftItem::Asset {
            data: NFTAssetData {
                collection: data.collection.clone(),
                asset: data.assets[0].clone(),
            },
        });

        assert_eq!(collection.title, data.collection.name);
        assert_eq!(collection.count_text.as_deref(), Some("2"));
        assert_eq!(collection.image_url, data.collection.images.preview.url);
        assert!(collection.is_verified);
        assert_eq!(asset.title, data.assets[0].name);
        assert_eq!(asset.count_text, None, "one asset is not a count");
        assert_eq!(asset.image_url, data.assets[0].images.preview.url);
        assert!(asset.is_verified, "an asset takes the verification of its collection");
    }

    #[test]
    fn test_can_send_needs_a_signing_wallet_that_still_holds_the_asset_on_a_transfer_chain() {
        assert!(can_send(&WalletType::Multicoin, Chain::Ethereum, true));
        assert!(!can_send(&WalletType::View, Chain::Ethereum, true));
        assert!(!can_send(&WalletType::Multicoin, Chain::Bitcoin, true));
        assert!(!can_send(&WalletType::Multicoin, Chain::Ethereum, false), "an asset the wallet sent away cannot be sent again");
    }

    #[test]
    fn test_unverified_collections_skip_verified_and_empty_ones() {
        let verified = NFTData::mock_with("verified", VerificationStatus::Verified, 1);
        let unverified = NFTData::mock_with("unverified", VerificationStatus::Unverified, 2);
        let empty = NFTData::mock_with("empty", VerificationStatus::Unverified, 0);

        assert_eq!(names(unverified_collections(vec![verified, unverified, empty])), vec!["unverified"]);
    }

    #[test]
    fn test_list_items_filter_by_verification_sort_and_show_a_lone_asset_as_itself() {
        let big = NFTData::mock_with("zebra", VerificationStatus::Verified, 3);
        let lone = NFTData::mock_with("alpha", VerificationStatus::Verified, 1);
        let unverified = NFTData::mock_with("beta", VerificationStatus::Unverified, 2);
        let unverified_lone = NFTData::mock_with("gamma", VerificationStatus::Suspicious, 1);
        let empty = NFTData::mock_with("empty", VerificationStatus::Verified, 0);
        let items = vec![lone, unverified_lone, empty, big, unverified];

        assert_eq!(labels(list_items(items.clone(), GemNftList::Collections)), vec!["collection zebra (3)", "asset alpha of alpha"]);
        assert_eq!(labels(list_items(items.clone(), GemNftList::Unverified)), vec!["collection beta (2)", "asset gamma of gamma"]);
        assert_eq!(
            labels(list_items(items, GemNftList::Avatar)),
            vec!["asset zebra of zebra", "asset zebra of zebra", "asset zebra of zebra", "asset alpha of alpha"],
            "an avatar is picked from the assets of the verified collections, in the same order"
        );
    }

    #[test]
    fn test_list_items_of_one_collection_are_its_assets() {
        let mut data = NFTData::mock_with("punks", VerificationStatus::Unverified, 2);
        data.assets[1].name = "punk 2".to_string();

        assert_eq!(
            labels(list_items(vec![data, NFTData::mock_with("empty", VerificationStatus::Verified, 0)], GemNftList::Collection)),
            vec!["asset punks of punks", "asset punk 2 of punks"]
        );
    }

    fn names(data: Vec<NFTData>) -> Vec<String> {
        data.into_iter().map(|item| item.collection.name).collect()
    }

    fn labels(items: Vec<GemNftItem>) -> Vec<String> {
        items
            .into_iter()
            .map(|item| match item {
                GemNftItem::Collection { data } => format!("collection {} ({})", data.collection.name, data.assets.len()),
                GemNftItem::Asset { data } => format!("asset {} of {}", data.asset.name, data.collection.name),
            })
            .collect()
    }

    #[test]
    fn test_search_matches_the_collection_name_or_its_assets() {
        let punks = NFTData::mock_with("Punks", VerificationStatus::Verified, 2);
        let mut apes = NFTData::mock_with("Apes", VerificationStatus::Verified, 3);
        apes.assets[1].name = "Punk Ape".to_string();
        apes.assets[2].name = "Ape 2".to_string();
        let lone = NFTData::mock_with("Punk Solo", VerificationStatus::Verified, 1);
        let items = vec![punks, apes, lone];

        assert_eq!(labels(search_collections(items.clone(), " punk ")), vec!["asset Punk Ape of Apes", "collection Punks (2)", "asset Punk Solo of Punk Solo"]);
        assert!(search_collections(items.clone(), "").is_empty());
        assert!(search_collections(items, "zzz").is_empty());
    }

    #[test]
    fn test_collectible_details_list_only_the_sections_the_asset_has() {
        let verified = NFTAssetData::mock();
        let mut suspicious = verified.clone();
        suspicious.collection.status = VerificationStatus::Suspicious;
        suspicious.collection.links = vec![AssetLink::new("https://example.com", LinkType::Website)];
        suspicious.asset.attributes = vec![NFTAttribute::new("Color", "Blue", NFTAttributeType::String)];

        assert_eq!(section_names(&collectible_details(&WalletType::Multicoin, &verified, true, None, None, false)), vec!["info"]);
        assert_eq!(section_names(&collectible_details(&WalletType::Multicoin, &suspicious, true, None, None, false)), vec!["status", "info", "attributes", "links"]);
    }

    #[test]
    fn test_the_collectible_menu_only_offers_saving_an_image_where_the_app_can() {
        let data = NFTAssetData::mock();

        assert_eq!(
            collectible_details(&WalletType::Multicoin, &data, true, None, None, false).actions,
            vec![GemCollectibleAction::SetAvatar, GemCollectibleAction::Refresh, GemCollectibleAction::Report]
        );
        assert_eq!(collectible_details(&WalletType::Multicoin, &data, true, None, None, true).actions.first(), Some(&GemCollectibleAction::SaveImage));
    }

    #[test]
    fn test_collectible_details_can_send_follows_the_wallet_and_ownership() {
        let data = NFTAssetData::mock();

        assert!(collectible_details(&WalletType::Multicoin, &data, true, None, None, false).can_send);
        assert!(!collectible_details(&WalletType::Multicoin, &data, false, None, None, false).can_send);
        assert!(!collectible_details(&WalletType::View, &data, true, None, None, false).can_send);
    }

    #[test]
    fn test_collectible_info_rows_hide_a_contract_that_is_empty_or_the_token_itself() {
        let mut data = NFTAssetData::mock();
        let link = BlockExplorerLink::mock();
        let rows = info_rows(&data, Some(link.clone()), Some(link.clone()));
        assert_eq!(
            rows,
            vec![
                GemListRow::Text {
                    title: GemListRowTitle::Collection,
                    value: data.collection.name.clone()
                },
                GemListRow::Network {
                    title: GemListRowTitle::Network,
                    chain: Chain::Ethereum,
                    name: "Ethereum".to_string()
                },
                GemListRow::Identifier {
                    title: GemListRowTitle::Contract,
                    copy: GemCopy {
                        kind: GemCopyKind::Address { chain: Chain::Ethereum },
                        value: data.collection.contract_address.clone(),
                        display: "0xdAC17...31ec7".to_string(),
                    },
                    explorer: Some(link.clone()),
                },
                GemListRow::Identifier {
                    title: GemListRowTitle::TokenId,
                    copy: GemCopy {
                        kind: GemCopyKind::Plain,
                        value: "1".to_string(),
                        display: "#1".to_string(),
                    },
                    explorer: Some(link),
                },
            ]
        );

        data.collection.contract_address = String::new();
        assert_eq!(row_names(&info_rows(&data, None, None)), vec!["collection", "network", "token_id"]);

        let ton = NFTAssetData {
            collection: NFTCollection {
                contract_address: NFTAsset::mock_ton().token_id,
                ..NFTCollection::mock()
            },
            asset: NFTAsset::mock_ton(),
        };
        assert_eq!(row_names(&info_rows(&ton, None, None)), vec!["collection", "network", "token_id"]);
    }

    #[test]
    fn test_collectible_token_id_reads_as_a_number_unless_it_is_address_sized() {
        let token_text = |data: NFTAssetData| match info_rows(&data, None, None).pop() {
            Some(GemListRow::Identifier { title: GemListRowTitle::TokenId, copy, .. }) => copy.display,
            row => panic!("expected a token id row, got {row:?}"),
        };

        assert_eq!(
            token_text(NFTAssetData {
                asset: NFTAsset {
                    token_id: "123".to_string(),
                    ..NFTAsset::mock()
                },
                ..NFTAssetData::mock()
            }),
            "#123"
        );
        assert_eq!(
            token_text(NFTAssetData {
                asset: NFTAsset {
                    token_id: "1234567890123456".to_string(),
                    ..NFTAsset::mock()
                },
                ..NFTAssetData::mock()
            }),
            "#1234567890123456"
        );
        assert_eq!(
            token_text(NFTAssetData {
                asset: NFTAsset {
                    token_id: "1234567890123456789".to_string(),
                    ..NFTAsset::mock()
                },
                ..NFTAssetData::mock()
            }),
            "1234567...56789"
        );
    }

    #[test]
    fn test_collectible_attributes_keep_text_and_parse_timestamps() {
        let text = attribute(&NFTAttribute::new("Length", "9", NFTAttributeType::String));
        let date = attribute(&NFTAttribute::new("Created", "1662714817", NFTAttributeType::Timestamp));
        let iso = attribute(&NFTAttribute::new("Created", "2022-09-09T09:13:37Z", NFTAttributeType::Timestamp));
        let broken = attribute(&NFTAttribute::new("Created", "soon", NFTAttributeType::Timestamp));
        let expected = DateTime::from_timestamp(1662714817, 0).unwrap();

        assert_eq!(text.value, GemCollectibleAttributeValue::Text { value: "9".to_string() });
        assert_eq!(date.value, GemCollectibleAttributeValue::Date { date: expected });
        assert_eq!(iso.value, GemCollectibleAttributeValue::Date { date: expected });
        assert_eq!(broken.value, GemCollectibleAttributeValue::Text { value: "soon".to_string() });
    }

    fn section_names(details: &GemCollectibleDetails) -> Vec<&'static str> {
        details
            .sections
            .iter()
            .map(|section| match section {
                GemCollectibleSection::Status { .. } => "status",
                GemCollectibleSection::Info { .. } => "info",
                GemCollectibleSection::Attributes { .. } => "attributes",
                GemCollectibleSection::Links { .. } => "links",
            })
            .collect()
    }

    fn row_names(rows: &[GemListRow]) -> Vec<&'static str> {
        rows.iter()
            .map(|row| match row {
                GemListRow::Text { .. } => "collection",
                GemListRow::Network { .. } => "network",
                GemListRow::Identifier { title: GemListRowTitle::Contract, .. } => "contract",
                GemListRow::Identifier { .. } => "token_id",
                _ => "other",
            })
            .collect()
    }

    #[test]
    fn test_collections_sort_by_size_then_name() {
        let big = NFTData::mock_with("zebra", VerificationStatus::Verified, 3);
        let small_a = NFTData::mock_with("alpha", VerificationStatus::Verified, 1);
        let small_b = NFTData::mock_with("beta", VerificationStatus::Verified, 1);

        let sorted = sorted_collections(vec![small_b, big, small_a]);

        assert_eq!(names(sorted), vec!["zebra", "alpha", "beta"]);
    }
}
