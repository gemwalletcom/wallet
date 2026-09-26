use std::iter::once;

use primitives::{AddressDetails, AddressName, AddressType, Asset, AssetBalance, AssetId, Chain, VerificationStatus, block_explorer::BlockExplorerLink};

use super::model::GemAddressDetails;
use crate::config::image::GemImage;
use crate::formatted_number::GemValueTone;
use crate::models::copy::address_copy;
use crate::models::list::{GemListRow, GemListRowTitle, GemListSection, GemListSectionFooter, GemListSectionTitle, GemNoticeKind, suspicious_address_notice};
use crate::models::state::{GemLoad, GemLoadState};
use crate::services::assets::icon::asset_icon;
use crate::services::assets::rules::asset_text;
use crate::services::balance::rules::{BalanceKind, balance_amount, balance_updates};
use crate::services::balance::{GemAssetBalance, GemBalanceRow};
use crate::services::contact::model::contact_avatar;
use crate::services::error::GemServiceError;
use crate::services::localization::GemLocalizedText;
use crate::services::wallet::model::GemWalletPlaceholder;

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

pub(super) fn refreshed(details: GemAddressDetails, remote: Result<AddressDetails, GemServiceError>) -> GemAddressDetails {
    match remote {
        Ok(AddressDetails { name, address_type, status, balances, .. }) => {
            let load = details.load().data(Ok(balances.map(|balances| balance_rows(details.chain, balances.coin, balances.staking)).unwrap_or_default()));
            GemAddressDetails {
                name,
                address_type: Some(address_type),
                status,
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

pub(super) fn sections(details: &GemAddressDetails, address_name: Option<&AddressName>) -> Vec<GemListSection> {
    let chain = details.chain;
    let asset = Asset::from_chain(chain);
    let address_name = address_name.filter(|address_name| !address_name.name.is_empty());
    let name = display_name(address_name.map(|address_name| address_name.name.clone()).or_else(|| details.name.clone()), &details.address);
    let address_type = address_name.map(|address_name| address_name.address_type.clone()).or_else(|| details.address_type.clone());
    let warning = match details.status {
        VerificationStatus::Suspicious => Some(section(vec![suspicious_address_notice(GemNoticeKind::Warning)])),
        VerificationStatus::Verified | VerificationStatus::Unverified => None,
    };
    let header = header(details, address_name);
    let info = name
        .into_iter()
        .map(|name| GemListRow::Text { title: GemListRowTitle::Name, value: name })
        .chain(address_type.into_iter().map(|address_type| GemListRow::Label {
            title: GemListRowTitle::Type,
            text: GemLocalizedText::AddressType { address_type },
            tone: GemValueTone::Plain,
            info: None,
            progress: false,
        }))
        .chain(once(GemListRow::Text {
            title: GemListRowTitle::Network,
            value: asset_text(&asset).network_full_name,
        }))
        .collect();
    let balances = balance_section(details, &asset);
    let balances = (!balances.is_empty()).then_some(GemListSection {
        title: GemListSectionTitle::Balances,
        footer: GemListSectionFooter::None,
        rows: balances,
    });
    once(section(vec![header]))
        .chain(warning)
        .chain([
            section(vec![GemListRow::Address {
                address: details.address.clone(),
                copy: details.copy.clone(),
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

fn header(details: &GemAddressDetails, address_name: Option<&AddressName>) -> GemListRow {
    let chain = details.chain;
    let chain_icon = GemListRow::Icon {
        icon: asset_icon(&AssetId::from_chain(chain)),
        image_url: None,
    };
    match address_name.map(|address_name| address_name.address_type.clone()).or_else(|| details.address_type.clone()) {
        Some(AddressType::InternalWallet) => GemListRow::WalletAvatar {
            image_url: address_name.and_then(|address_name| address_name.image_url.clone()).filter(|url| !url.is_empty()),
            placeholder: GemWalletPlaceholder::Multicoin,
        },
        Some(AddressType::Contact) => contact_avatar(address_name, None).map_or(chain_icon, |avatar| GemListRow::Avatar { avatar }),
        Some(AddressType::Asset) => GemListRow::Icon {
            icon: asset_icon(&AssetId::from_token(chain, &details.address)),
            image_url: None,
        },
        Some(AddressType::Validator) => GemListRow::Icon {
            icon: asset_icon(&AssetId::from_chain(chain)),
            image_url: Some(
                GemImage::Validator {
                    chain,
                    validator_id: details.address.clone(),
                }
                .url(),
            ),
        },
        Some(AddressType::Address | AddressType::Contract) | None => chain_icon,
    }
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

fn balance_rows(chain: Chain, coin: AssetBalance, stake: Option<AssetBalance>) -> Vec<GemBalanceRow> {
    let balance = balance_updates(once((BalanceKind::Coin, coin)).chain(stake.map(|stake| (BalanceKind::Stake, stake))).collect())
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
        GemLoadState::NoData | GemLoadState::Data | GemLoadState::Error { .. } => details
            .balances
            .iter()
            .map(|row| GemListRow::Amount {
                title: row.title(),
                amount: balance_amount(&row.value(), asset),
                info: None,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;

    use super::*;
    use crate::formatted_number::GemFormattedNumber;
    use crate::precision::GemValueStyle;
    use crate::services::contact::model::GemAvatar;

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
            sections(&details, None)[3].rows,
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
                address_type: AddressType::Validator,
                balances: None,
                ..AddressDetails::mock()
            }),
        );

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
    fn test_the_address_name_names_the_address() {
        let loading = details(Chain::Ethereum, "0x1".to_string(), BlockExplorerLink::mock(), GemLoad::loading());
        let flagged_contract = GemAddressDetails {
            name: Some("Tether USD".to_string()),
            address_type: Some(AddressType::Contract),
            status: VerificationStatus::Suspicious,
            state: GemLoadState::Data,
            ..loading.clone()
        };
        let contact = AddressName {
            image_url: Some("avatar.png".to_string()),
            ..AddressName::mock("0x1", "John Smith", AddressType::Contact, VerificationStatus::Verified)
        };
        let wallet = AddressName::mock("0x1", "Savings", AddressType::InternalWallet, VerificationStatus::Verified);
        let identity = |sections: &[GemListSection]| (sections[0].rows.clone(), sections[2].rows[..2].to_vec());
        let named = |name: &str, address_type: AddressType| {
            vec![
                GemListRow::Text {
                    title: GemListRowTitle::Name,
                    value: name.to_string(),
                },
                GemListRow::Label {
                    title: GemListRowTitle::Type,
                    text: GemLocalizedText::AddressType { address_type },
                    tone: GemValueTone::Plain,
                    info: None,
                    progress: false,
                },
            ]
        };
        let avatar = GemListRow::Avatar {
            avatar: GemAvatar {
                image_url: Some("avatar.png".to_string()),
                initials: "JO".to_string(),
            },
        };

        assert_eq!(identity(&sections(&loading, Some(&contact))), (vec![avatar.clone()], named("John Smith", AddressType::Contact)));
        let wallet_with_image = AddressName {
            image_url: Some("savings.png".to_string()),
            ..wallet.clone()
        };
        let wallet_avatar = |image_url: Option<&str>| GemListRow::WalletAvatar {
            image_url: image_url.map(str::to_string),
            placeholder: GemWalletPlaceholder::Multicoin,
        };
        assert_eq!(
            identity(&sections(&loading, Some(&wallet_with_image))),
            (vec![wallet_avatar(Some("savings.png"))], named("Savings", AddressType::InternalWallet))
        );
        assert_eq!(identity(&sections(&loading, Some(&wallet))), (vec![wallet_avatar(None)], named("Savings", AddressType::InternalWallet)));
        let flagged = sections(&flagged_contract, Some(&contact));
        let without_warning = |sections: &[GemListSection]| [&sections[..1], &sections[2..]].concat();
        assert_eq!(flagged[1].rows, vec![suspicious_address_notice(GemNoticeKind::Warning)]);
        assert_eq!(identity(&without_warning(&flagged)), (vec![avatar], named("John Smith", AddressType::Contact)));
        assert_eq!(identity(&without_warning(&sections(&flagged_contract, None))).1, named("Tether USD", AddressType::Contract));
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
        let failed = GemAddressDetails {
            state: GemLoadState::Error {
                error: GemServiceError::Gateway { msg: "offline".to_string() },
            },
            ..loading.clone()
        };
        let titles = |details: &GemAddressDetails| sections(details, None).into_iter().map(|section| section.title).collect::<Vec<_>>();
        let without_balances = vec![GemListSectionTitle::None; 4];

        assert_eq!(
            sections(&validator, None)[2].rows,
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
        assert_eq!(titles(&validator), without_balances);
        assert_eq!(titles(&failed), without_balances);
        assert_eq!(sections(&loading, None)[3].rows, vec![GemListRow::Loading]);
        assert_eq!(
            sections(&loading, None)[0].rows,
            vec![GemListRow::Icon {
                icon: asset_icon(&AssetId::from_chain(Chain::Cosmos)),
                image_url: None
            }]
        );
    }

    #[test]
    fn test_a_token_or_validator_header_shows_its_logo() {
        let loaded = |chain: Chain, address: &str, address_type: AddressType| GemAddressDetails {
            address_type: Some(address_type),
            state: GemLoadState::Data,
            ..details(chain, address.to_string(), BlockExplorerLink::mock(), GemLoad::loading())
        };
        let token = loaded(Chain::Ethereum, "0xdAC17F958D2ee523a2206206994597C13D831ec7", AddressType::Asset);
        let validator = loaded(Chain::Cosmos, "cosmosvaloper1", AddressType::Validator);
        let contract = loaded(Chain::Ethereum, "0x1", AddressType::Contract);

        assert_eq!(
            sections(&token, None)[0].rows,
            vec![GemListRow::Icon {
                icon: asset_icon(&AssetId::from_token(Chain::Ethereum, "0xdAC17F958D2ee523a2206206994597C13D831ec7")),
                image_url: None,
            }]
        );
        assert_eq!(
            sections(&validator, None)[0].rows,
            vec![GemListRow::Icon {
                icon: asset_icon(&AssetId::from_chain(Chain::Cosmos)),
                image_url: Some("https://assets.gemwallet.com/blockchains/cosmos/validators/cosmosvaloper1/logo.png".to_string()),
            }]
        );
        assert_eq!(
            sections(&contract, None)[0].rows,
            vec![GemListRow::Icon {
                icon: asset_icon(&AssetId::from_chain(Chain::Ethereum)),
                image_url: None,
            }]
        );
    }

    #[test]
    fn test_display_name() {
        assert_eq!(display_name(None, "0x1"), None);
        assert_eq!(display_name(Some(String::new()), "0x1"), None);
        assert_eq!(display_name(Some("0x1".to_string()), "0x1"), None);
        assert_eq!(display_name(Some("Main Wallet".to_string()), "0x1"), Some("Main Wallet".to_string()));
    }

    #[test]
    fn test_the_address_row_shows_the_full_address_and_copies_it() {
        let address = "0x1234567890abcdef1234567890abcdef12345678".to_string();
        let details = details(Chain::Ethereum, address.clone(), BlockExplorerLink::mock(), GemLoad::loading());

        assert_eq!(
            sections(&details, None)[1].rows,
            vec![GemListRow::Address {
                address: address.clone(),
                copy: address_copy(Chain::Ethereum, address),
            }]
        );
    }
}
