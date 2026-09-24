use std::iter::once;

use primitives::{AddressDetails, AddressType, Asset, AssetBalance, AssetId, Chain, VerificationStatus, block_explorer::BlockExplorerLink};

use super::model::GemAddressDetails;
use crate::formatted_number::GemValueTone;
use crate::models::copy::address_copy;
use crate::models::list::{GemListRow, GemListRowTitle, GemListSection, GemListSectionFooter, GemListSectionTitle, GemNoticeKind};
use crate::models::state::{GemLoad, GemLoadState};
use crate::services::assets::rules::asset_text;
use crate::services::balance::rules::{balance_amount, balance_updates, chain_balances};
use crate::services::balance::{GemAssetBalance, GemBalanceRow};
use crate::services::error::GemServiceError;
use crate::services::localization::GemLocalizedText;

const BALANCES_UNAVAILABLE: &str = "balances unavailable";

pub(super) fn details(chain: Chain, address: String, link: BlockExplorerLink, balances: GemLoad<Vec<GemBalanceRow>>) -> GemAddressDetails {
    GemAddressDetails {
        chain,
        copy: address_copy(chain, address.clone()),
        address,
        name: None,
        address_type: None,
        status: VerificationStatus::Unverified,
        link,
        state: balances.state,
        balances: balances.value,
    }
}

pub(super) fn refreshed(details: GemAddressDetails, result: Result<AddressDetails, GemServiceError>) -> GemAddressDetails {
    match result {
        Ok(remote) => {
            let balances = match (remote.balances, &remote.address_type) {
                (Some(balances), _) => Ok(balance_rows(details.chain, balances.coin, balances.staking)),
                (None, AddressType::Validator) => Ok(Vec::new()),
                (None, AddressType::Address | AddressType::Contract | AddressType::Contact | AddressType::InternalWallet) => Err(GemServiceError::Gateway { msg: BALANCES_UNAVAILABLE.to_string() }),
            };
            let load = details.load().data(balances);
            GemAddressDetails {
                name: display_name(remote.name, &details.address),
                address_type: Some(remote.address_type),
                status: remote.status,
                state: load.state,
                balances: load.value,
                ..details
            }
        }
        Err(error) => {
            let load = details.load().data(Err(error));
            GemAddressDetails {
                state: load.state,
                balances: load.value,
                ..details
            }
        }
    }
}

pub(super) fn sections(details: &GemAddressDetails) -> Vec<GemListSection> {
    let chain = details.chain;
    let asset = Asset::from_chain(chain);
    let warning = match details.status {
        VerificationStatus::Suspicious => Some(section(vec![GemListRow::Notice {
            title: GemListRowTitle::Warning,
            message: Some(GemLocalizedText::SuspiciousAddress),
            kind: GemNoticeKind::Error,
        }])),
        VerificationStatus::Verified | VerificationStatus::Unverified => None,
    };
    let balances = match details.address_type {
        Some(AddressType::Validator) => None,
        Some(AddressType::Address | AddressType::Contract | AddressType::Contact | AddressType::InternalWallet) | None => Some(GemListSection {
            title: GemListSectionTitle::Balances,
            footer: GemListSectionFooter::None,
            rows: balance_section(details, &asset),
        }),
    };
    let info = details
        .name
        .iter()
        .map(|name| GemListRow::Text {
            title: GemListRowTitle::Name,
            value: name.clone(),
        })
        .chain(details.address_type.iter().map(|address_type| GemListRow::Label {
            title: GemListRowTitle::Type,
            text: GemLocalizedText::AddressType { address_type: address_type.clone() },
            tone: GemValueTone::Plain,
            info: None,
            progress: false,
        }))
        .chain(once(GemListRow::Text {
            title: GemListRowTitle::Network,
            value: asset_text(&asset).network_full_name,
        }))
        .collect();
    warning
        .into_iter()
        .chain([
            section(vec![GemListRow::Icon { chain }]),
            section(vec![GemListRow::Identifier {
                title: GemListRowTitle::Address,
                copy: details.copy.clone(),
                explorer: None,
            }]),
            section(info),
        ])
        .chain(balances)
        .chain(once(section(vec![GemListRow::Explorer {
            name: details.link.name.clone(),
            url: details.link.link.clone(),
        }])))
        .collect()
}

fn section(rows: Vec<GemListRow>) -> GemListSection {
    GemListSection {
        title: GemListSectionTitle::None,
        footer: GemListSectionFooter::None,
        rows,
    }
}

fn display_name(name: Option<String>, address: &str) -> Option<String> {
    name.filter(|name| !name.is_empty() && name != address)
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
                amount: balance_amount(&row.value(), asset),
                info: None,
            })
            .collect(),
        GemLoadState::Error { error } => vec![GemListRow::Error { error: error.clone() }],
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;

    use super::*;
    use crate::formatted_number::GemFormattedNumber;
    use crate::precision::GemValueStyle;

    #[test]
    fn test_balance_rows() {
        let ethereum = AssetId::from_chain(Chain::Ethereum);
        let cosmos = AssetId::from_chain(Chain::Cosmos);

        assert_eq!(balance_rows(Chain::Ethereum, AssetBalance::new(ethereum, BigUint::ZERO), None), vec![GemBalanceRow::Available { value: BigUint::ZERO }]);
        assert_eq!(
            balance_rows(
                Chain::Cosmos,
                AssetBalance::new(cosmos.clone(), BigUint::from(5u32)),
                Some(AssetBalance::new_staking(cosmos.clone(), BigUint::from(100u32), BigUint::ZERO, BigUint::from(1u32))),
            ),
            vec![GemBalanceRow::Available { value: BigUint::from(5u32) }, GemBalanceRow::Staked { value: BigUint::from(101u32) },]
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
    fn test_refreshed() {
        let loading = details(Chain::Ethereum, "0x1".to_string(), BlockExplorerLink::mock(), GemLoad::loading());
        let unavailable = GemLoadState::Error {
            error: GemServiceError::Gateway { msg: BALANCES_UNAVAILABLE.to_string() },
        };
        let offline = GemServiceError::Gateway { msg: "offline".to_string() };
        let contract = refreshed(
            loading.clone(),
            Ok(AddressDetails {
                name: Some("Tether USD".to_string()),
                address_type: AddressType::Contract,
                status: VerificationStatus::Verified,
                ..AddressDetails::mock()
            }),
        );
        let validator = refreshed(
            loading.clone(),
            Ok(AddressDetails {
                name: Some("Stakin".to_string()),
                address_type: AddressType::Validator,
                balances: None,
                ..AddressDetails::mock()
            }),
        );
        let failed_balances = refreshed(loading.clone(), Ok(AddressDetails { balances: None, ..AddressDetails::mock() }));

        assert_eq!(
            contract,
            GemAddressDetails {
                name: Some("Tether USD".to_string()),
                address_type: Some(AddressType::Contract),
                status: VerificationStatus::Verified,
                state: GemLoadState::Data,
                balances: vec![GemBalanceRow::Available {
                    value: BigUint::from(1_500_000_000_000_000_000u64),
                }],
                ..loading.clone()
            }
        );
        assert_eq!((validator.state, validator.balances), (GemLoadState::Data, Vec::new()));
        assert_eq!((failed_balances.address_type, failed_balances.state), (Some(AddressType::Address), unavailable));
        assert_eq!(refreshed(contract.clone(), Err(offline.clone())), contract);
        assert_eq!(
            refreshed(loading.clone(), Err(offline.clone())),
            GemAddressDetails {
                state: GemLoadState::Error { error: offline },
                ..loading
            }
        );
    }

    #[test]
    fn test_sections() {
        let loading = details(Chain::Cosmos, "cosmosvaloper1".to_string(), BlockExplorerLink::mock(), GemLoad::loading());
        let validator = GemAddressDetails {
            name: Some("Stakin".to_string()),
            address_type: Some(AddressType::Validator),
            state: GemLoadState::Data,
            ..loading.clone()
        };
        let suspicious = GemAddressDetails {
            address_type: Some(AddressType::Address),
            status: VerificationStatus::Suspicious,
            ..loading.clone()
        };
        let titles = |details: &GemAddressDetails| sections(details).into_iter().map(|section| section.title).collect::<Vec<_>>();

        assert_eq!(
            sections(&validator)[2].rows,
            vec![
                GemListRow::Text {
                    title: GemListRowTitle::Name,
                    value: "Stakin".to_string()
                },
                GemListRow::Label {
                    title: GemListRowTitle::Type,
                    text: GemLocalizedText::AddressType { address_type: AddressType::Validator },
                    tone: GemValueTone::Plain,
                    info: None,
                    progress: false,
                },
                GemListRow::Text {
                    title: GemListRowTitle::Network,
                    value: "Cosmos".to_string(),
                },
            ]
        );
        assert_eq!(titles(&validator), vec![GemListSectionTitle::None; 4]);
        assert_eq!(titles(&loading)[3], GemListSectionTitle::Balances);
        assert_eq!(
            sections(&suspicious)[0].rows,
            vec![GemListRow::Notice {
                title: GemListRowTitle::Warning,
                message: Some(GemLocalizedText::SuspiciousAddress),
                kind: GemNoticeKind::Error,
            }]
        );
        assert_eq!(sections(&loading)[0].rows, vec![GemListRow::Icon { chain: Chain::Cosmos }]);
    }

    #[test]
    fn test_display_name() {
        assert_eq!(display_name(None, "0x1"), None);
        assert_eq!(display_name(Some(String::new()), "0x1"), None);
        assert_eq!(display_name(Some("0x1".to_string()), "0x1"), None);
        assert_eq!(display_name(Some("Main Wallet".to_string()), "0x1"), Some("Main Wallet".to_string()));
    }

    #[test]
    fn test_the_address_row_copies_the_address() {
        let address = "0x1234567890abcdef1234567890abcdef12345678".to_string();
        let details = details(Chain::Ethereum, address.clone(), BlockExplorerLink::mock(), GemLoad::loading());

        assert_eq!(
            sections(&details)[1].rows,
            vec![GemListRow::Identifier {
                title: GemListRowTitle::Address,
                copy: address_copy(Chain::Ethereum, address),
                explorer: None,
            }]
        );
    }
}
