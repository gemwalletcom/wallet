use primitives::{AssetId, Chain, ChainAsset, Platform, StakeChain};

use super::model::{GemInfoAction, GemInfoAmount, GemInfoDescription, GemInfoImage, GemInfoSheet, GemInfoTitle};
use crate::config::docs::DocsUrl;
use crate::formatted_number::GemFormattedNumber;
use crate::models::list::GemInfoTopic;
use crate::precision::GemValueStyle;
use crate::services::assets::icon::{GemAssetIcon, asset_icon};
use crate::services::confirm::error::{GemConfirmErrorInfo, GemConfirmErrorSheet};

pub fn sheet(topic: &GemInfoTopic, platform: Platform) -> GemInfoSheet {
    let learn_more = |docs: DocsUrl| Some(GemInfoAction::LearnMore { url: docs.url_for(platform) });
    match topic {
        GemInfoTopic::NetworkFee { asset } => info_sheet(
            GemInfoTitle::NetworkFee,
            GemInfoDescription::NetworkFee {
                network: network_name(asset.chain()),
                symbol: asset.symbol.clone(),
            },
            GemInfoImage::NetworkFee,
            learn_more(DocsUrl::NetworkFees),
        ),
        GemInfoTopic::MinimumAmount { asset, minimum } => info_sheet(
            GemInfoTitle::MinimumAmount,
            GemInfoDescription::MinimumAmount {
                network: network_name(asset.chain()),
                amount: GemFormattedNumber::asset_amount(minimum, asset, GemValueStyle::Full),
            },
            GemInfoImage::Logo,
            Some(GemInfoAction::Buy { symbol: asset.symbol.clone() }),
        ),
        GemInfoTopic::NoQuote => info_sheet(GemInfoTitle::NoQuote, GemInfoDescription::NoQuote, GemInfoImage::Logo, learn_more(DocsUrl::NoQuotes)),
        GemInfoTopic::PriceImpact => info_sheet(GemInfoTitle::PriceImpact, GemInfoDescription::PriceImpact, GemInfoImage::Logo, learn_more(DocsUrl::PriceImpact)),
        GemInfoTopic::Slippage => info_sheet(GemInfoTitle::Slippage, GemInfoDescription::Slippage, GemInfoImage::Logo, learn_more(DocsUrl::Slippage)),
        GemInfoTopic::OpenInterest => info_sheet(GemInfoTitle::OpenInterest, GemInfoDescription::OpenInterest, GemInfoImage::Logo, learn_more(DocsUrl::PerpetualsOpenInterest)),
        GemInfoTopic::FundingApr => info_sheet(GemInfoTitle::FundingApr, GemInfoDescription::FundingApr, GemInfoImage::Logo, learn_more(DocsUrl::PerpetualsFundingRate)),
        GemInfoTopic::StakeApr { chain } => info_sheet(GemInfoTitle::Apr, GemInfoDescription::Apr, chain_logo(*chain), learn_more(DocsUrl::StakingAPR)),
        GemInfoTopic::StakeLockTime { chain } => info_sheet(GemInfoTitle::LockTime, GemInfoDescription::LockTime, chain_logo(*chain), learn_more(DocsUrl::StakingLockTime)),
        GemInfoTopic::StakeFrozenRequired => info_sheet(GemInfoTitle::StakeFrozenRequired, GemInfoDescription::StakeFrozenRequired, GemInfoImage::Logo, learn_more(DocsUrl::Staking(StakeChain::Tron))),
        GemInfoTopic::TransactionStatus { state, tone, icon } => info_sheet(
            GemInfoTitle::TransactionState { state: *state },
            GemInfoDescription::TransactionState { tone: *tone },
            GemInfoImage::TransactionState { icon: icon.clone(), tone: *tone },
            learn_more(DocsUrl::TransactionStatus),
        ),
        GemInfoTopic::AutoClose => info_sheet(GemInfoTitle::AutoClose, GemInfoDescription::AutoClose, GemInfoImage::Logo, learn_more(DocsUrl::PerpetualsAutoclose)),
        GemInfoTopic::LiquidationPrice => info_sheet(GemInfoTitle::LiquidationPrice, GemInfoDescription::LiquidationPrice, GemInfoImage::Logo, learn_more(DocsUrl::PerpetualsLiquidationPrice)),
        GemInfoTopic::FundingPayments => info_sheet(GemInfoTitle::FundingPayments, GemInfoDescription::FundingPayments, GemInfoImage::Logo, learn_more(DocsUrl::PerpetualsFundingPayments)),
        GemInfoTopic::FullyDilutedValuation => info_sheet(GemInfoTitle::FullyDilutedValuation, GemInfoDescription::FullyDilutedValuation, GemInfoImage::Logo, None),
        GemInfoTopic::CirculatingSupply => info_sheet(GemInfoTitle::CirculatingSupply, GemInfoDescription::CirculatingSupply, GemInfoImage::Logo, None),
        GemInfoTopic::TotalSupply => info_sheet(GemInfoTitle::TotalSupply, GemInfoDescription::TotalSupply, GemInfoImage::Logo, None),
        GemInfoTopic::MaxSupply => info_sheet(GemInfoTitle::MaxSupply, GemInfoDescription::MaxSupply, GemInfoImage::Logo, None),
        GemInfoTopic::EstimatedConfirmation { chain } => info_sheet(GemInfoTitle::EstimatedConfirmation, GemInfoDescription::EstimatedConfirmation { network: network_name(*chain) }, GemInfoImage::NetworkFee, None),
        GemInfoTopic::WatchWallet => info_sheet(GemInfoTitle::WatchWallet, GemInfoDescription::WatchWallet, GemInfoImage::WatchWallet, learn_more(DocsUrl::WhatIsWatchWallet)),
        GemInfoTopic::PaymentVerification => info_sheet(GemInfoTitle::PaymentVerification, GemInfoDescription::PaymentVerification, GemInfoImage::Logo, None),
        GemInfoTopic::StakingReservedFees { asset } => info_sheet(
            GemInfoTitle::StakingReservedFees,
            GemInfoDescription::StakingReservedFees,
            GemInfoImage::Asset { icon: asset_icon(&asset.id) },
            learn_more(DocsUrl::NetworkFees),
        ),
        GemInfoTopic::PendingUnconfirmedBalance => info_sheet(GemInfoTitle::Pending, GemInfoDescription::Pending, GemInfoImage::Logo, None),
        GemInfoTopic::AssetStatus { status } => info_sheet(
            GemInfoTitle::AssetStatus { status: *status },
            GemInfoDescription::AssetStatus { status: *status },
            GemInfoImage::AssetStatus { status: *status },
            learn_more(DocsUrl::TokenVerification),
        ),
        GemInfoTopic::ExistingWalletImported { name } => info_sheet(GemInfoTitle::WalletName { name: name.clone() }, GemInfoDescription::ExistingWalletImported, GemInfoImage::Logo, Some(GemInfoAction::Continue)),
    }
}

pub fn confirm_error_sheet(info: &GemConfirmErrorInfo, platform: Platform) -> GemInfoSheet {
    let acquire = info.asset.clone().zip(info.acquire.clone()).map(|(asset, acquire)| GemInfoAction::Acquire { asset, acquire });
    let asset_image = info.asset.as_ref().map_or(GemInfoImage::Logo, |asset| GemInfoImage::Asset { icon: asset_icon(&asset.id) });
    let required = info.required.clone().map(|amount| GemInfoAmount { amount, fiat: info.required_fiat.clone() });
    let balance_required = GemInfoTitle::BalanceRequired { symbol: info.title.clone() };
    match &info.sheet {
        GemConfirmErrorSheet::BalanceRequired => info_sheet(
            balance_required,
            GemInfoDescription::BalanceRequired {
                required: info.required.clone(),
                available: info.available.clone(),
                shortfall: info.shortfall.clone(),
            },
            asset_image,
            acquire,
        ),
        GemConfirmErrorSheet::NetworkFeeRequired | GemConfirmErrorSheet::NetworkFeeMissing => info_sheet(
            balance_required,
            match &info.available {
                Some(available) => GemInfoDescription::InsufficientNetworkFeeBalance {
                    required,
                    network: info.asset.as_ref().map_or_else(|| info.title.clone(), |asset| network_name(asset.chain())),
                    available: available.clone(),
                    shortfall: info.shortfall.clone(),
                },
                None => GemInfoDescription::InsufficientNetworkFee { title: info.title.clone() },
            },
            asset_image,
            acquire,
        ),
        GemConfirmErrorSheet::MinimumAccountBalance => info_sheet(
            GemInfoTitle::AccountMinimumBalance,
            GemInfoDescription::AccountMinimumBalance { amount: info.required.clone() },
            GemInfoImage::Logo,
            Some(GemInfoAction::LearnMore {
                url: DocsUrl::AccountMinimalBalance.url_for(platform),
            }),
        ),
        GemConfirmErrorSheet::SwapMinimum { provider, provider_name } => info_sheet(
            GemInfoTitle::MinimumAmount,
            GemInfoDescription::SwapMinimumAmount {
                provider: provider_name.clone(),
                required,
                available: info.available.clone(),
                shortfall: info.shortfall.clone(),
            },
            GemInfoImage::SwapProvider { provider: *provider },
            acquire,
        ),
        GemConfirmErrorSheet::DustThreshold { chain } => info_sheet(
            GemInfoTitle::TransferError,
            GemInfoDescription::DustThreshold { network: network_name(*chain) },
            GemInfoImage::Asset {
                icon: asset_icon(&AssetId::from_chain(*chain)),
            },
            Some(GemInfoAction::LearnMore { url: DocsUrl::Dust.url_for(platform) }),
        ),
        GemConfirmErrorSheet::Malicious => info_sheet(GemInfoTitle::MaliciousTransaction, GemInfoDescription::MaliciousTransaction, GemInfoImage::Logo, None),
        GemConfirmErrorSheet::MemoRequired { symbol } => info_sheet(GemInfoTitle::Warning, GemInfoDescription::MemoRequired { symbol: symbol.clone() }, GemInfoImage::Logo, None),
    }
}

fn info_sheet(title: GemInfoTitle, description: GemInfoDescription, image: GemInfoImage, action: Option<GemInfoAction>) -> GemInfoSheet {
    GemInfoSheet { title, description, image, action }
}

fn network_name(chain: Chain) -> String {
    ChainAsset::from_chain(chain).network_name
}

fn chain_logo(chain: Chain) -> GemInfoImage {
    GemInfoImage::Asset {
        icon: GemAssetIcon {
            badge: None,
            ..asset_icon(&AssetId::from_chain(chain))
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::confirm::GemAcquireAssetFlow;
    use crate::services::confirm::error::{GemConfirmErrorDisplay, GemConfirmRequirement};
    use crate::services::confirm::rules::error_info;
    use crate::services::transactions::GemTransactionStateTone;
    use primitives::{Asset, Currency, SwapProvider, TransactionState, VerificationStatus};

    #[test]
    fn test_a_sheet_shows_its_docs_link_always_and_other_actions_only_when_handled() {
        let learn_more = GemInfoSheet {
            action: Some(GemInfoAction::LearnMore { url: "https://docs".to_string() }),
            ..sheet(&GemInfoTopic::PriceImpact, Platform::IOS)
        };
        let proceed = GemInfoSheet {
            action: Some(GemInfoAction::Continue),
            ..learn_more.clone()
        };

        assert_eq!(learn_more.button(false), learn_more.action);
        assert_eq!(proceed.button(false), None, "nothing would run the action");
        assert_eq!(proceed.button(true), Some(GemInfoAction::Continue));
        assert_eq!(GemInfoSheet { action: None, ..proceed }.button(true), None);
    }

    #[test]
    fn test_a_network_fee_sheet_names_the_network_and_links_the_platform_docs() {
        let ethereum = Asset::from_chain(Chain::Ethereum);
        let sheet = sheet(&GemInfoTopic::NetworkFee { asset: ethereum.clone() }, Platform::IOS);

        assert_eq!(
            sheet.description,
            GemInfoDescription::NetworkFee {
                network: network_name(Chain::Ethereum),
                symbol: ethereum.symbol,
            }
        );
        assert_eq!(sheet.image, GemInfoImage::NetworkFee);
        assert_eq!(
            sheet.action,
            Some(GemInfoAction::LearnMore {
                url: DocsUrl::NetworkFees.url_for(Platform::IOS)
            })
        );
    }

    #[test]
    fn test_a_minimum_amount_offers_to_buy_the_asset_and_shows_the_full_amount() {
        let bitcoin = Asset::from_chain(Chain::Bitcoin);
        let sheet = sheet(
            &GemInfoTopic::MinimumAmount {
                asset: bitcoin.clone(),
                minimum: 12_345.into(),
            },
            Platform::Android,
        );

        let GemInfoDescription::MinimumAmount { amount, .. } = sheet.description else {
            panic!("expected the minimum amount description");
        };
        assert_eq!(amount.exact.as_deref(), Some("0.00012345"));
        assert_eq!(sheet.action, Some(GemInfoAction::Buy { symbol: bitcoin.symbol }));
    }

    #[test]
    fn test_the_stake_sheets_show_the_chain_logo_without_a_badge() {
        let GemInfoImage::Asset { icon } = sheet(&GemInfoTopic::StakeApr { chain: Chain::Base }, Platform::IOS).image else {
            panic!("expected the chain logo");
        };
        assert_eq!(icon.badge, None);
    }

    #[test]
    fn test_a_transaction_status_sheet_reads_the_state_and_badges_the_icon_with_its_tone() {
        let icon = asset_icon(&AssetId::from_chain(Chain::Solana));
        let sheet = sheet(
            &GemInfoTopic::TransactionStatus {
                state: TransactionState::Failed,
                tone: GemTransactionStateTone::Error,
                icon: icon.clone(),
            },
            Platform::IOS,
        );

        assert_eq!(sheet.title, GemInfoTitle::TransactionState { state: TransactionState::Failed });
        assert_eq!(sheet.image, GemInfoImage::TransactionState { icon, tone: GemTransactionStateTone::Error });
    }

    #[test]
    fn test_the_market_and_pending_sheets_have_no_button() {
        for topic in [GemInfoTopic::FullyDilutedValuation, GemInfoTopic::MaxSupply, GemInfoTopic::PendingUnconfirmedBalance, GemInfoTopic::PaymentVerification] {
            assert_eq!(sheet(&topic, Platform::IOS).action, None, "{topic:?}");
        }
        assert_eq!(
            sheet(&GemInfoTopic::AssetStatus { status: VerificationStatus::Suspicious }, Platform::IOS).action,
            Some(GemInfoAction::LearnMore {
                url: DocsUrl::TokenVerification.url_for(Platform::IOS)
            })
        );
    }

    fn confirm_sheet(display: GemConfirmErrorDisplay) -> GemInfoSheet {
        let ethereum = Asset::from_chain(Chain::Ethereum).id;
        let info = error_info(&display, &[], Currency::USD, &ethereum, &ethereum).expect("a sheet");
        confirm_error_sheet(&info, Platform::IOS)
    }

    fn requirement() -> GemConfirmRequirement {
        GemConfirmRequirement {
            required: GemFormattedNumber::amount(2.0, Some("ETH".to_string()), GemValueStyle::Full),
            available: GemFormattedNumber::amount(1.0, Some("ETH".to_string()), GemValueStyle::Full),
            shortfall: GemFormattedNumber::amount(1.0, Some("ETH".to_string()), GemValueStyle::Full),
        }
    }

    #[test]
    fn test_a_balance_shortfall_offers_to_acquire_the_asset_it_names() {
        let ethereum = Asset::from_chain(Chain::Ethereum);
        let sheet = confirm_sheet(GemConfirmErrorDisplay::BalanceRequired {
            asset: ethereum.clone(),
            requirement: requirement(),
        });

        assert_eq!(sheet.title, GemInfoTitle::BalanceRequired { symbol: ethereum.symbol.clone() });
        assert!(matches!(sheet.action, Some(GemInfoAction::Acquire { ref asset, ref acquire }) if *asset == ethereum && acquire.flow == GemAcquireAssetFlow::Fiat));
    }

    #[test]
    fn test_a_missing_network_fee_balance_reads_the_short_explanation() {
        let ethereum = Asset::from_chain(Chain::Ethereum);
        let missing = confirm_sheet(GemConfirmErrorDisplay::NetworkFeeMissing {
            asset: ethereum.clone(),
            title: "Ethereum".to_string(),
        });
        let short = confirm_sheet(GemConfirmErrorDisplay::NetworkFeeRequired {
            asset: ethereum,
            title: "Ethereum".to_string(),
            requirement: requirement(),
        });

        assert_eq!(missing.description, GemInfoDescription::InsufficientNetworkFee { title: "Ethereum".to_string() });
        assert!(matches!(short.description, GemInfoDescription::InsufficientNetworkFeeBalance { ref network, .. } if *network == network_name(Chain::Ethereum)));
    }

    #[test]
    fn test_a_swap_minimum_shows_the_provider_and_the_dust_sheet_links_its_docs() {
        let swap = confirm_sheet(GemConfirmErrorDisplay::SwapMinimum {
            asset: Asset::from_chain(Chain::Ethereum),
            provider: SwapProvider::NearIntents,
            provider_name: "NEAR Intents".to_string(),
            requirement: requirement(),
        });
        let dust = confirm_sheet(GemConfirmErrorDisplay::DustThreshold { chain: Chain::Bitcoin });

        assert_eq!(swap.image, GemInfoImage::SwapProvider { provider: SwapProvider::NearIntents });
        assert_eq!(dust.action, Some(GemInfoAction::LearnMore { url: DocsUrl::Dust.url_for(Platform::IOS) }));
    }
}
