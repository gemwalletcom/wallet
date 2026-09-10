use chrono::{DateTime, Utc};
use primitives::{Account, AddressFormatStyle, BlockExplorerLink, Chain, NFTAssetData, NFTAttribute, NFTAttributeType, NFTData, VerificationStatus, Wallet, WalletType};

use super::model::{
    GemCollectibleAttribute, GemCollectibleAttributeValue, GemCollectibleDetails, GemCollectibleIdentifier, GemCollectibleRow, GemCollectibleSection,
};
use crate::address_formatter::format_address;
use crate::config::chain::supports_nft_transfer;
use crate::services::chain::rules::chain_matches_query;

const TOKEN_ID_ADDRESS_LENGTH: usize = 16;

pub fn verified_collections(data: Vec<NFTData>) -> Vec<NFTData> {
    collections(data, true)
}

pub fn unverified_collections(data: Vec<NFTData>) -> Vec<NFTData> {
    collections(data, false)
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemNftSearchItem {
    Collection { data: NFTData },
    Asset { data: NFTAssetData },
}

pub fn search_collections(data: Vec<NFTData>, query: &str) -> Vec<GemNftSearchItem> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    sorted_collections(data)
        .into_iter()
        .flat_map(|data| {
            if data.collection.name.to_lowercase().contains(&query) {
                return vec![GemNftSearchItem::Collection { data }];
            }
            let mut assets: Vec<_> = data.assets.into_iter().filter(|asset| asset.name.to_lowercase().contains(&query)).collect();
            assets.sort_by_key(|asset| asset.name.to_lowercase());
            assets
                .into_iter()
                .map(|asset| GemNftSearchItem::Asset {
                    data: NFTAssetData {
                        collection: data.collection.clone(),
                        asset,
                    },
                })
                .collect()
        })
        .collect()
}

pub fn sorted_collections(data: Vec<NFTData>) -> Vec<NFTData> {
    let mut sorted = data;
    sorted.sort_by(|left, right| {
        right
            .assets
            .len()
            .cmp(&left.assets.len())
            .then_with(|| left.collection.name.to_lowercase().cmp(&right.collection.name.to_lowercase()))
    });
    sorted
}

pub fn nft_chains() -> Vec<Chain> {
    Chain::all().into_iter().filter(Chain::is_nft_supported).collect()
}

pub fn receive_accounts(wallet: &Wallet, query: &str) -> Vec<Account> {
    wallet
        .accounts
        .iter()
        .filter(|account| account.chain.is_nft_supported() && chain_matches_query(account.chain, query))
        .cloned()
        .collect()
}

pub fn can_send(wallet_type: &WalletType, chain: Chain, is_owned: bool) -> bool {
    *wallet_type != WalletType::View && supports_nft_transfer(chain) && is_owned
}

pub fn collectible_details(
    wallet_type: &WalletType,
    data: &NFTAssetData,
    is_owned: bool,
    contract_explorer: Option<BlockExplorerLink>,
    token_explorer: Option<BlockExplorerLink>,
) -> GemCollectibleDetails {
    let status = (data.collection.status != VerificationStatus::Verified).then_some(GemCollectibleSection::Status {
        status: data.collection.status,
    });
    let info = GemCollectibleSection::Info {
        rows: info_rows(data, contract_explorer, token_explorer),
    };
    let attributes = (!data.asset.attributes.is_empty()).then(|| GemCollectibleSection::Attributes {
        attributes: data.asset.attributes.iter().map(attribute).collect(),
    });
    let links = (!data.collection.links.is_empty()).then(|| GemCollectibleSection::Links {
        links: data.collection.links.clone(),
    });
    GemCollectibleDetails {
        can_send: can_send(wallet_type, data.asset.chain, is_owned),
        sections: [status, Some(info), attributes, links].into_iter().flatten().collect(),
    }
}

fn info_rows(data: &NFTAssetData, contract_explorer: Option<BlockExplorerLink>, token_explorer: Option<BlockExplorerLink>) -> Vec<GemCollectibleRow> {
    let chain = data.asset.chain;
    let token_id = &data.asset.token_id;
    let contract = &data.collection.contract_address;
    let contract_row = (!contract.is_empty() && contract != token_id).then(|| GemCollectibleRow::Contract {
        identifier: GemCollectibleIdentifier {
            value: contract.clone(),
            text: format_address(contract, Some(chain), AddressFormatStyle::Short),
            explorer: contract_explorer,
        },
    });
    let token_text = if token_id.chars().count() > TOKEN_ID_ADDRESS_LENGTH {
        format_address(token_id, Some(chain), AddressFormatStyle::Short)
    } else {
        format!("#{token_id}")
    };
    [
        Some(GemCollectibleRow::Collection {
            name: data.collection.name.clone(),
        }),
        Some(GemCollectibleRow::Network { chain }),
        contract_row,
        Some(GemCollectibleRow::TokenId {
            identifier: GemCollectibleIdentifier {
                value: token_id.clone(),
                text: token_text,
                explorer: token_explorer,
            },
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
        value: date.map(|date| GemCollectibleAttributeValue::Date { date }).unwrap_or_else(|| GemCollectibleAttributeValue::Text {
            value: attribute.value.clone(),
        }),
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
    data.into_iter()
        .filter(|item| !item.assets.is_empty() && (item.collection.status == VerificationStatus::Verified) == verified)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetLink, LinkType, NFTAsset, NFTCollection, NFTData, Wallet};

    #[test]
    fn test_receive_accounts_keeps_nft_chains_matching_the_query() {
        let wallet = Wallet::mock_with_accounts(Account::mock_chains(&[Chain::Ethereum, Chain::Bitcoin, Chain::Solana], "address"));
        let chains = |query: &str| receive_accounts(&wallet, query).into_iter().map(|account| account.chain).collect::<Vec<_>>();
        assert_eq!(chains(""), vec![Chain::Ethereum, Chain::Solana]);
        assert_eq!(chains("sol"), vec![Chain::Solana]);
    }

    #[test]
    fn test_can_send_needs_a_signing_wallet_that_still_holds_the_asset_on_a_transfer_chain() {
        assert!(can_send(&WalletType::Multicoin, Chain::Ethereum, true));
        assert!(!can_send(&WalletType::View, Chain::Ethereum, true));
        assert!(!can_send(&WalletType::Multicoin, Chain::Bitcoin, true));
        assert!(
            !can_send(&WalletType::Multicoin, Chain::Ethereum, false),
            "an asset the wallet sent away cannot be sent again"
        );
    }

    #[test]
    fn test_collections_split_by_verification_and_skip_empty_ones() {
        let verified = NFTData::mock_with("verified", VerificationStatus::Verified, 1);
        let unverified = NFTData::mock_with("unverified", VerificationStatus::Unverified, 2);
        let empty = NFTData::mock_with("empty", VerificationStatus::Verified, 0);
        let items = vec![verified, unverified, empty];

        assert_eq!(names(verified_collections(items.clone())), vec!["verified"]);
        assert_eq!(names(unverified_collections(items)), vec!["unverified"]);
    }

    fn names(data: Vec<NFTData>) -> Vec<String> {
        data.into_iter().map(|item| item.collection.name).collect()
    }

    #[test]
    fn test_search_matches_the_collection_name_or_its_assets() {
        let punks = NFTData::mock_with("Punks", VerificationStatus::Verified, 2);
        let mut apes = NFTData::mock_with("Apes", VerificationStatus::Verified, 3);
        apes.assets[1].name = "Punk Ape".to_string();
        apes.assets[2].name = "Ape 2".to_string();
        let items = vec![punks.clone(), apes.clone()];

        let results = search_collections(items.clone(), " punk ");
        let labels: Vec<String> = results
            .iter()
            .map(|item| match item {
                GemNftSearchItem::Collection { data } => format!("collection {} ({})", data.collection.name, data.assets.len()),
                GemNftSearchItem::Asset { data } => format!("asset {} of {}", data.asset.name, data.collection.name),
            })
            .collect();
        assert_eq!(labels, vec!["asset Punk Ape of Apes", "collection Punks (2)"]);
        assert!(search_collections(items.clone(), "").is_empty());
        assert!(search_collections(items, "zzz").is_empty());
    }

    #[test]
    fn test_collectible_details_list_only_the_sections_the_asset_has() {
        let verified = NFTAssetData {
            collection: NFTCollection::mock(),
            asset: NFTAsset::mock(),
        };
        let mut suspicious = verified.clone();
        suspicious.collection.status = VerificationStatus::Suspicious;
        suspicious.collection.links = vec![AssetLink::new("https://example.com", LinkType::Website)];
        suspicious.asset.attributes = vec![NFTAttribute::new("Color", "Blue", NFTAttributeType::String)];

        assert_eq!(
            section_names(&collectible_details(&WalletType::Multicoin, &verified, true, None, None)),
            vec!["info"]
        );
        assert_eq!(
            section_names(&collectible_details(&WalletType::Multicoin, &suspicious, true, None, None)),
            vec!["status", "info", "attributes", "links"]
        );
    }

    #[test]
    fn test_collectible_details_can_send_follows_the_wallet_and_ownership() {
        let data = NFTAssetData {
            collection: NFTCollection::mock(),
            asset: NFTAsset::mock(),
        };

        assert!(collectible_details(&WalletType::Multicoin, &data, true, None, None).can_send);
        assert!(!collectible_details(&WalletType::Multicoin, &data, false, None, None).can_send);
        assert!(!collectible_details(&WalletType::View, &data, true, None, None).can_send);
    }

    #[test]
    fn test_collectible_info_rows_hide_a_contract_that_is_empty_or_the_token_itself() {
        let mut data = NFTAssetData {
            collection: NFTCollection::mock(),
            asset: NFTAsset::mock(),
        };
        let link = BlockExplorerLink::mock();
        let rows = info_rows(&data, Some(link.clone()), Some(link.clone()));
        assert_eq!(
            rows,
            vec![
                GemCollectibleRow::Collection {
                    name: data.collection.name.clone()
                },
                GemCollectibleRow::Network { chain: Chain::Ethereum },
                GemCollectibleRow::Contract {
                    identifier: GemCollectibleIdentifier {
                        value: data.collection.contract_address.clone(),
                        text: "0xdAC17...31ec7".to_string(),
                        explorer: Some(link.clone()),
                    }
                },
                GemCollectibleRow::TokenId {
                    identifier: GemCollectibleIdentifier {
                        value: "1".to_string(),
                        text: "#1".to_string(),
                        explorer: Some(link),
                    }
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
        let token_text = |token_id: &str| {
            let data = NFTAssetData {
                collection: NFTCollection::mock(),
                asset: NFTAsset {
                    token_id: token_id.to_string(),
                    ..NFTAsset::mock()
                },
            };
            match info_rows(&data, None, None).pop() {
                Some(GemCollectibleRow::TokenId { identifier }) => identifier.text,
                row => panic!("expected a token id row, got {row:?}"),
            }
        };

        assert_eq!(token_text("123"), "#123");
        assert_eq!(token_text("1234567890123456"), "#1234567890123456");
        assert_eq!(token_text("1234567890123456789"), "1234567...56789");
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

    fn row_names(rows: &[GemCollectibleRow]) -> Vec<&'static str> {
        rows.iter()
            .map(|row| match row {
                GemCollectibleRow::Collection { .. } => "collection",
                GemCollectibleRow::Network { .. } => "network",
                GemCollectibleRow::Contract { .. } => "contract",
                GemCollectibleRow::TokenId { .. } => "token_id",
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
