use std::iter::once;

use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{AddressName, Asset, AssetBalance, AssetId, Chain, block_explorer::BlockExplorerLink};

use super::model::GemAddressDetails;
use crate::formatted_number::GemFormattedNumber;
use crate::models::copy::address_copy;
use crate::models::list::{GemListRow, GemListRowTitle, GemListSection, GemListSectionFooter, GemListSectionTitle};
use crate::models::state::{GemLoad, GemLoadState};
use crate::precision::GemValueStyle;
use crate::services::assets::rules::asset_text;
use crate::services::balance::rules::{balance_updates, chain_balances};
use crate::services::balance::{GemAssetBalance, GemBalanceRow};

pub(super) fn details(chain: Chain, address: String, name: Option<String>, link: BlockExplorerLink, balances: GemLoad<Vec<GemBalanceRow>>) -> GemAddressDetails {
    GemAddressDetails {
        chain,
        copy: address_copy(chain, address.clone()),
        address,
        name,
        link,
        state: balances.state,
        balances: balances.value,
    }
}

pub(super) fn sections(details: &GemAddressDetails) -> Vec<GemListSection> {
    let chain = details.chain;
    let asset = Asset::from_chain(chain);
    vec![
        GemListSection {
            title: GemListSectionTitle::None,
            footer: GemListSectionFooter::None,
            rows: vec![GemListRow::Icon { chain }],
        },
        GemListSection {
            title: GemListSectionTitle::None,
            footer: GemListSectionFooter::None,
            rows: vec![GemListRow::Address {
                address: details.address.clone(),
                copy: details.copy.clone(),
            }],
        },
        GemListSection {
            title: GemListSectionTitle::None,
            footer: GemListSectionFooter::None,
            rows: details
                .name
                .iter()
                .map(|name| GemListRow::Text {
                    title: GemListRowTitle::Name,
                    value: name.clone(),
                })
                .chain(once(GemListRow::Text {
                    title: GemListRowTitle::Network,
                    value: asset_text(&asset).network_full_name,
                }))
                .collect(),
        },
        GemListSection {
            title: GemListSectionTitle::Balances,
            footer: GemListSectionFooter::None,
            rows: balance_section(details, &asset),
        },
        GemListSection {
            title: GemListSectionTitle::None,
            footer: GemListSectionFooter::None,
            rows: vec![GemListRow::Explorer {
                name: details.link.name.clone(),
                url: details.link.link.clone(),
            }],
        },
    ]
}

pub(super) fn display_name(name: Option<AddressName>, address: &str) -> Option<String> {
    name.map(|name| name.name).filter(|name| !name.is_empty() && name != address)
}

pub(super) fn balance_rows(chain: Chain, coin: AssetBalance, stake: Option<AssetBalance>) -> Vec<GemBalanceRow> {
    let balance = balance_updates(chain_balances(vec![coin], stake.into_iter().collect(), Vec::new(), Vec::new()))
        .iter()
        .fold(GemAssetBalance::zero(AssetId::from_chain(chain)), |balance, update| balance.applying(update));
    let breakdown = balance.detail_rows(chain, false).into_iter().filter(|row| match row {
        GemBalanceRow::Available { .. } => false,
        GemBalanceRow::Staked { .. } | GemBalanceRow::Earn { .. } | GemBalanceRow::PendingUnconfirmed { .. } | GemBalanceRow::Reserved { .. } => true,
    });
    once(GemBalanceRow::Available { value: balance.available.clone() }).chain(breakdown).collect()
}

fn balance_section(details: &GemAddressDetails, asset: &Asset) -> Vec<GemListRow> {
    match &details.state {
        GemLoadState::Loading => vec![GemListRow::Loading],
        GemLoadState::NoData | GemLoadState::Data => details
            .balances
            .iter()
            .map(|row| GemListRow::Amount {
                title: row.title(),
                amount: amount(&row.value(), asset),
                info: None,
            })
            .collect(),
        GemLoadState::Error { error } => vec![GemListRow::Error { error: error.clone() }],
    }
}

fn amount(value: &BigUint, asset: &Asset) -> GemFormattedNumber {
    let value = BigNumberFormatter::value_as_f64(&value.to_string(), asset.decimals.unsigned_abs()).unwrap_or_default();
    GemFormattedNumber::amount(value, Some(asset.symbol.clone()), GemValueStyle::Auto)
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use primitives::{AddressType, VerificationStatus};

    use super::*;
    use crate::precision::GemValueStyle;

    #[test]
    fn test_balance_rows() {
        let ethereum = AssetId::from_chain(Chain::Ethereum);
        let cosmos = AssetId::from_chain(Chain::Cosmos);

        assert_eq!(
            balance_rows(Chain::Ethereum, AssetBalance::new(ethereum, BigUint::ZERO), None),
            vec![GemBalanceRow::Available { value: BigUint::ZERO }]
        );
        assert_eq!(
            balance_rows(
                Chain::Cosmos,
                AssetBalance::new(cosmos.clone(), BigUint::from(5u32)),
                Some(AssetBalance::new_staking(cosmos.clone(), BigUint::from(100u32), BigUint::ZERO, BigUint::from(1u32))),
            ),
            vec![
                GemBalanceRow::Available { value: BigUint::from(5u32) },
                GemBalanceRow::Staked { value: BigUint::from(101u32) },
            ]
        );
        assert_eq!(
            balance_rows(
                Chain::Cosmos,
                AssetBalance::new(cosmos.clone(), BigUint::from(5u32)),
                Some(AssetBalance::new_staking(cosmos, BigUint::ZERO, BigUint::ZERO, BigUint::ZERO)),
            ),
            vec![GemBalanceRow::Available { value: BigUint::from(5u32) }]
        );
    }

    #[test]
    fn test_sections_print_every_balance_at_the_asset_decimals() {
        let details = details(
            Chain::Ethereum,
            "0x1".to_string(),
            None,
            BlockExplorerLink::mock(),
            GemLoad {
                state: GemLoadState::Data,
                value: vec![GemBalanceRow::Available {
                    value: BigUint::from(1_500_000_000_000_000_000u64),
                }],
            },
        );

        assert_eq!(
            sections(&details)[3].rows,
            vec![GemListRow::Amount {
                title: GemListRowTitle::Available,
                amount: GemFormattedNumber::amount(1.5, Some("ETH".to_string()), GemValueStyle::Auto),
                info: None,
            }]
        );
    }

    #[test]
    fn test_a_known_name_is_a_row_of_its_own_above_the_network() {
        let named = details(
            Chain::Ethereum,
            "0x1".to_string(),
            Some("Main Wallet".to_string()),
            BlockExplorerLink::mock(),
            GemLoad::loading(),
        );
        let unnamed = details(Chain::Ethereum, "0x1".to_string(), None, BlockExplorerLink::mock(), GemLoad::loading());
        let network = GemListRow::Text {
            title: GemListRowTitle::Network,
            value: "Ethereum".to_string(),
        };

        assert_eq!(
            sections(&named)[2].rows,
            vec![
                GemListRow::Text {
                    title: GemListRowTitle::Name,
                    value: "Main Wallet".to_string()
                },
                network.clone(),
            ]
        );
        assert_eq!(sections(&unnamed)[2].rows, vec![network]);
    }

    #[test]
    fn test_display_name() {
        assert_eq!(display_name(None, "0x1"), None);
        assert_eq!(
            display_name(Some(AddressName::mock("0x1", "", AddressType::Address, VerificationStatus::Unverified)), "0x1"),
            None
        );
        assert_eq!(
            display_name(Some(AddressName::mock("0x1", "0x1", AddressType::Address, VerificationStatus::Unverified)), "0x1"),
            None
        );
        assert_eq!(
            display_name(Some(AddressName::mock("0x1", "Main Wallet", AddressType::Address, VerificationStatus::Unverified)), "0x1"),
            Some("Main Wallet".to_string())
        );
    }

    #[test]
    fn test_the_header_row_carries_the_address_to_copy() {
        let address = "0x1234567890abcdef1234567890abcdef12345678".to_string();
        let details = details(Chain::Ethereum, address.clone(), None, BlockExplorerLink::mock(), GemLoad::loading());

        assert_eq!(
            sections(&details)[1].rows,
            vec![GemListRow::Address {
                address: address.clone(),
                copy: address_copy(Chain::Ethereum, address.clone()),
            }]
        );
    }
}
