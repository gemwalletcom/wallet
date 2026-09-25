use primitives::{Asset, AssetId, Banner, BannerEvent, BannerState, Chain, ChainAsset, Platform, VerificationStatus, Wallet, WalletSource, WalletType};

use super::model::{GemBannerButton, GemBannerContent, GemBannerContext, GemBannerDescription, GemBannerDestination, GemBannerIcon, GemBannerItem, GemBannerKey, GemBannerStyle, GemBannerTitle};
use crate::config::chain::account_activation_fee_url;
use crate::config::docs::DocsUrl;
use crate::formatted_number::GemFormattedNumber;
#[cfg(test)]
use crate::models::custom_types::GemBigInt;
use crate::precision::GemValueStyle;
use crate::services::transfer::rules as transfer_rules;

const ACCOUNT_ACTIVATION_CHAINS: [Chain; 3] = [Chain::Xrp, Chain::Stellar, Chain::Algorand];
const TRADE_PERPETUALS_CHAINS: [Chain; 2] = [Chain::HyperCore, Chain::Hyperliquid];

fn is_visible(state: BannerState) -> bool {
    match state {
        BannerState::Active | BannerState::AlwaysActive => true,
        BannerState::Cancelled => false,
    }
}

pub fn default_state(event: BannerEvent) -> BannerState {
    match event {
        BannerEvent::ActivateAsset | BannerEvent::AccountBlockedMultiSignature => BannerState::AlwaysActive,
        BannerEvent::Stake | BannerEvent::AccountActivation | BannerEvent::SuspiciousAsset | BannerEvent::Onboarding | BannerEvent::TradePerpetuals => BannerState::Active,
    }
}

fn asset_key(chain: Chain, event: BannerEvent) -> GemBannerKey {
    GemBannerKey {
        wallet_id: None,
        asset_id: Some(AssetId::from_chain(chain)),
        event,
    }
}

pub fn setup_keys() -> Vec<GemBannerKey> {
    Chain::all()
        .into_iter()
        .filter(Chain::is_stake_supported)
        .map(|chain| asset_key(chain, BannerEvent::Stake))
        .chain(TRADE_PERPETUALS_CHAINS.into_iter().map(|chain| asset_key(chain, BannerEvent::TradePerpetuals)))
        .collect()
}

pub fn wallet_setup_keys(wallet: &Wallet) -> Vec<GemBannerKey> {
    let onboarding = match wallet.source {
        WalletSource::Create => Some(GemBannerKey {
            wallet_id: Some(wallet.id.clone()),
            asset_id: None,
            event: BannerEvent::Onboarding,
        }),
        WalletSource::Import => None,
    };
    ACCOUNT_ACTIVATION_CHAINS.into_iter().map(|chain| asset_key(chain, BannerEvent::AccountActivation)).chain(onboarding).collect()
}

fn is_visible_event(event: BannerEvent, context: &GemBannerContext) -> bool {
    let has_asset = context.asset.is_some();
    let can_sign = context.wallet.as_ref().is_some_and(|wallet| wallet.wallet_type != WalletType::View);
    match event {
        BannerEvent::AccountBlockedMultiSignature => true,
        BannerEvent::AccountActivation => can_sign && (!has_asset || !context.has_available_balance),
        BannerEvent::Stake => can_sign && has_asset && !context.has_stake_balance,
        BannerEvent::ActivateAsset => can_sign && has_asset && !context.is_asset_activated,
        BannerEvent::SuspiciousAsset => has_asset && is_suspicious(context),
        BannerEvent::TradePerpetuals => has_asset && context.wallet.as_ref().is_some_and(|wallet| crate::services::perpetual::rules::supports_perpetuals(wallet.wallet_type, &wallet.chains())),
        BannerEvent::Onboarding => !has_asset && context.is_wallet_empty,
    }
}

pub fn banner_content(event: BannerEvent, asset: Option<&Asset>, state: BannerState, platform: Platform) -> GemBannerContent {
    let style = match event {
        BannerEvent::Onboarding => GemBannerStyle::Welcome,
        _ => GemBannerStyle::List,
    };
    GemBannerContent {
        icon: banner_icon(event, asset.map(|asset| asset.id.chain)),
        title: banner_title(event, asset),
        description: banner_description(event, asset),
        destination: banner_destination(event, asset, platform),
        buttons: match style {
            GemBannerStyle::Welcome => vec![GemBannerButton::Buy, GemBannerButton::Receive],
            GemBannerStyle::List => Vec::new(),
        },
        can_close: state != BannerState::AlwaysActive,
        style,
    }
}

fn banner_destination(event: BannerEvent, asset: Option<&Asset>, platform: Platform) -> Option<GemBannerDestination> {
    let url = |url| Some(GemBannerDestination::Url { url });
    match event {
        BannerEvent::Stake => Some(GemBannerDestination::Stake),
        BannerEvent::ActivateAsset => Some(GemBannerDestination::ActivateAsset {
            transfer: transfer_rules::activate_asset_transfer_data(asset?.clone()),
        }),
        BannerEvent::TradePerpetuals => Some(GemBannerDestination::Perpetuals),
        BannerEvent::AccountActivation => url(account_activation_fee_url(asset?.id.chain)?),
        BannerEvent::AccountBlockedMultiSignature => url(DocsUrl::ExternallyControlledAccount.url_for(platform)),
        BannerEvent::SuspiciousAsset => url(DocsUrl::TokenVerification.url_for(platform)),
        BannerEvent::Onboarding => None,
    }
}

fn banner_icon(event: BannerEvent, chain: Option<Chain>) -> Option<GemBannerIcon> {
    match event {
        BannerEvent::Stake => Some(GemBannerIcon::MoneyBag),
        BannerEvent::AccountActivation | BannerEvent::ActivateAsset => chain.map(|chain| GemBannerIcon::Network { chain }),
        BannerEvent::AccountBlockedMultiSignature => Some(GemBannerIcon::Warning),
        BannerEvent::SuspiciousAsset => Some(GemBannerIcon::Suspicious),
        BannerEvent::Onboarding => Some(GemBannerIcon::Bitcoin),
        BannerEvent::TradePerpetuals => Some(GemBannerIcon::Perpetuals),
    }
}

fn banner_title(event: BannerEvent, asset: Option<&Asset>) -> Option<GemBannerTitle> {
    match event {
        BannerEvent::Stake => Some(GemBannerTitle::Stake { asset_name: asset?.name.clone() }),
        BannerEvent::AccountActivation => Some(GemBannerTitle::AccountActivation),
        BannerEvent::AccountBlockedMultiSignature => Some(GemBannerTitle::Warning),
        BannerEvent::ActivateAsset => Some(GemBannerTitle::ActivateAsset),
        BannerEvent::SuspiciousAsset => Some(GemBannerTitle::SuspiciousAsset),
        BannerEvent::Onboarding => Some(GemBannerTitle::Onboarding),
        BannerEvent::TradePerpetuals => Some(GemBannerTitle::TradePerpetuals),
    }
}

fn banner_description(event: BannerEvent, asset: Option<&Asset>) -> Option<GemBannerDescription> {
    match event {
        BannerEvent::Stake => Some(GemBannerDescription::Stake { asset_symbol: asset?.symbol.clone() }),
        BannerEvent::AccountActivation => {
            let asset = asset?;
            Some(GemBannerDescription::AccountActivation {
                network_name: network_name(asset.id.chain),
                fee: GemFormattedNumber::asset_amount(&num_bigint::BigInt::from(asset.id.chain.account_activation_fee()?), asset, GemValueStyle::Auto),
            })
        }
        BannerEvent::AccountBlockedMultiSignature => Some(GemBannerDescription::ExternallyControlledAccount { network_name: network_name(asset?.id.chain) }),
        BannerEvent::ActivateAsset => {
            let asset = asset?;
            Some(GemBannerDescription::ActivateAsset {
                asset_symbol: asset.symbol.clone(),
                network_name: network_name(asset.id.chain),
            })
        }
        BannerEvent::SuspiciousAsset => Some(GemBannerDescription::SuspiciousAsset),
        BannerEvent::Onboarding => Some(GemBannerDescription::Onboarding),
        BannerEvent::TradePerpetuals => Some(GemBannerDescription::TradePerpetuals),
    }
}

fn network_name(chain: Chain) -> String {
    ChainAsset::from_chain(chain).network_name
}

pub(super) fn visible_banners(stored: Vec<Banner>, context: &GemBannerContext) -> Vec<Banner> {
    let asset_id = context.asset_id();
    let mut banners: Vec<GemBannerItem> = Vec::new();
    for item in stored.iter().map(banner_item).chain(extra_banners(asset_id.clone())) {
        let applies = match &asset_id {
            Some(asset_id) => item.applies_to_asset(asset_id),
            None => item.applies_to_wallet(),
        };
        if !applies {
            continue;
        }
        if banners.iter().any(|existing| existing.event == item.event) {
            continue;
        }
        if is_visible(item.state) && is_visible_event(item.event, context) {
            banners.push(item);
        }
    }
    banners.sort_by_key(|item| (state_priority(item.state), event_priority(item.event)));
    banners
        .into_iter()
        .map(|item| match stored.iter().find(|banner| banner.event == item.event && banner.asset.as_ref().map(|asset| &asset.id) == item.asset_id.as_ref()) {
            Some(banner) => banner.clone(),
            None => context.banner(item),
        })
        .collect()
}

fn banner_item(banner: &Banner) -> GemBannerItem {
    GemBannerItem {
        event: banner.event,
        state: banner.state,
        asset_id: banner.asset.as_ref().map(|asset| asset.id.clone()),
    }
}

fn extra_banners(asset_id: Option<AssetId>) -> Vec<GemBannerItem> {
    [BannerEvent::ActivateAsset, BannerEvent::SuspiciousAsset]
        .into_iter()
        .map(|event| GemBannerItem {
            event,
            state: default_state(event),
            asset_id: asset_id.clone(),
        })
        .collect()
}

fn is_suspicious(context: &GemBannerContext) -> bool {
    context.asset_rank_score.is_some_and(|score| VerificationStatus::from_rank(score) == VerificationStatus::Suspicious)
}

fn state_priority(state: BannerState) -> u8 {
    match state {
        BannerState::AlwaysActive => 0,
        BannerState::Active => 1,
        BannerState::Cancelled => 2,
    }
}

fn event_priority(event: BannerEvent) -> u8 {
    match event {
        BannerEvent::AccountBlockedMultiSignature => 0,
        BannerEvent::AccountActivation => 1,
        BannerEvent::ActivateAsset => 2,
        BannerEvent::SuspiciousAsset => 3,
        BannerEvent::Onboarding => 4,
        BannerEvent::Stake => 5,
        BannerEvent::TradePerpetuals => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::super::model::{BannerScope, banner_scope};
    use super::*;
    use primitives::known_assets::TRON_USDT;
    use primitives::{AccountDataType, TransactionInputType};

    #[test]
    fn test_setup_keys() {
        let keys = setup_keys();
        assert!(keys.iter().all(|key| key.wallet_id.is_none()));
        assert!(keys.iter().any(|key| key.event == BannerEvent::Stake && key.asset_id == Some(AssetId::from_chain(Chain::Cosmos))));
        assert!(!keys.iter().any(|key| key.event == BannerEvent::Stake && key.asset_id == Some(AssetId::from_chain(Chain::Bitcoin))));
        assert_eq!(keys.iter().filter(|key| key.event == BannerEvent::TradePerpetuals).count(), 2);
    }

    #[test]
    fn test_wallet_setup_keys() {
        let mut wallet = Wallet::mock_with_accounts(vec![]);
        let imported = wallet_setup_keys(&wallet);
        assert_eq!(imported.len(), 3);
        assert!(imported.iter().all(|key| key.event == BannerEvent::AccountActivation));

        wallet.source = WalletSource::Create;
        let created = wallet_setup_keys(&wallet);
        assert_eq!(created.len(), 4);
        assert_eq!(created.last().map(|key| (key.event, key.wallet_id.clone())), Some((BannerEvent::Onboarding, Some(wallet.id))));
    }

    fn events(banners: &[Banner]) -> Vec<BannerEvent> {
        banners.iter().map(|banner| banner.event).collect()
    }

    #[test]
    fn test_visible_banners_asset_rules() {
        let stake = vec![Banner::mock(BannerEvent::Stake, BannerState::Active)];
        assert_eq!(events(&visible_banners(stake.clone(), &GemBannerContext::mock())), vec![BannerEvent::Stake]);
        let staked = GemBannerContext {
            has_stake_balance: true,
            ..GemBannerContext::mock()
        };
        assert!(visible_banners(stake, &staked).is_empty());

        let inactive = GemBannerContext {
            is_asset_activated: false,
            ..GemBannerContext::mock()
        };
        assert_eq!(events(&visible_banners(vec![], &inactive)), vec![BannerEvent::ActivateAsset]);
        assert!(visible_banners(vec![], &GemBannerContext::mock()).is_empty());

        let suspicious = GemBannerContext {
            asset_rank_score: Some(5),
            ..GemBannerContext::mock()
        };
        assert_eq!(events(&visible_banners(vec![], &suspicious)), vec![BannerEvent::SuspiciousAsset]);

        let activation = vec![Banner::mock(BannerEvent::AccountActivation, BannerState::AlwaysActive)];
        assert_eq!(events(&visible_banners(activation.clone(), &GemBannerContext::mock())), vec![BannerEvent::AccountActivation]);
        let funded = GemBannerContext {
            has_available_balance: true,
            ..GemBannerContext::mock()
        };
        assert!(visible_banners(activation, &funded).is_empty());

        let perpetuals = vec![Banner::mock(BannerEvent::TradePerpetuals, BannerState::Active)];
        assert_eq!(events(&visible_banners(perpetuals.clone(), &GemBannerContext::mock())), vec![BannerEvent::TradePerpetuals]);
        let unsupported = GemBannerContext {
            wallet: Some(Wallet::mock_with_chains(&[Chain::Ethereum])),
            ..GemBannerContext::mock()
        };
        assert!(visible_banners(perpetuals.clone(), &unsupported).is_empty());
        let no_wallet = GemBannerContext { wallet: None, ..GemBannerContext::mock() };
        assert!(visible_banners(perpetuals, &no_wallet).is_empty());
    }

    #[test]
    fn test_visible_banners_keep_warnings_without_signing_actions_for_view_wallets() {
        let context = GemBannerContext {
            wallet: Some(Wallet {
                wallet_type: WalletType::View,
                ..Wallet::mock()
            }),
            is_asset_activated: false,
            asset_rank_score: Some(5),
            ..GemBannerContext::mock()
        };
        let stored = vec![
            Banner::mock(BannerEvent::Stake, BannerState::Active),
            Banner::mock(BannerEvent::AccountActivation, BannerState::Active),
            Banner::mock(BannerEvent::AccountBlockedMultiSignature, BannerState::AlwaysActive),
        ];

        assert_eq!(events(&visible_banners(stored, &context)), vec![BannerEvent::AccountBlockedMultiSignature, BannerEvent::SuspiciousAsset]);
        assert_eq!(events(&visible_banners(vec![], &context)), vec![BannerEvent::SuspiciousAsset]);
    }

    #[test]
    fn test_the_wallet_screen_reads_only_the_banners_no_asset_owns() {
        let context = GemBannerContext {
            asset: None,
            is_wallet_empty: true,
            ..GemBannerContext::mock()
        };
        let stored = vec![
            Banner {
                asset: Some(Asset::from_chain(Chain::Ethereum)),
                ..Banner::mock(BannerEvent::Stake, BannerState::Active)
            },
            Banner {
                asset: Some(Asset::from_chain(Chain::Ethereum)),
                ..Banner::mock(BannerEvent::AccountActivation, BannerState::Active)
            },
            Banner {
                asset: Some(Asset::from_chain(Chain::Tron)),
                ..Banner::mock(BannerEvent::AccountBlockedMultiSignature, BannerState::AlwaysActive)
            },
            Banner {
                asset: None,
                ..Banner::mock(BannerEvent::Onboarding, BannerState::Active)
            },
        ];

        assert_eq!(events(&visible_banners(stored, &context)), vec![BannerEvent::AccountBlockedMultiSignature, BannerEvent::Onboarding]);
        let wallet_events: Vec<BannerEvent> = BannerEvent::all().into_iter().filter(|event| banner_scope(*event) != BannerScope::Asset).collect();
        assert_eq!(wallet_events, crate::constants::WALLET_BANNER_EVENTS, "the apps ask their stores for every banner that is not an asset's");
    }

    #[test]
    fn test_multi_signature_warning_for_every_wallet_type_and_scene() {
        let tron = Asset::from_chain(Chain::Tron);
        let token = TRON_USDT.clone();
        let warning = Banner {
            asset: Some(tron.clone()),
            ..Banner::mock(BannerEvent::AccountBlockedMultiSignature, BannerState::AlwaysActive)
        };
        for wallet_type in [WalletType::Multicoin, WalletType::Single, WalletType::PrivateKey, WalletType::View] {
            for asset in [None, Some(tron.clone()), Some(token.clone())] {
                let context = GemBannerContext {
                    wallet: Some(Wallet { wallet_type, ..Wallet::mock() }),
                    asset,
                    ..GemBannerContext::mock()
                };
                assert_eq!(visible_banners(vec![warning.clone()], &context), vec![warning.clone()], "{wallet_type:?}");
            }
        }
    }

    #[test]
    fn test_asset_scope_is_filtered_before_deduplication() {
        let token = Asset::mock_ethereum_usdc();
        let context = GemBannerContext {
            asset: Some(token.clone()),
            ..GemBannerContext::mock()
        };
        let warning = Banner::mock(BannerEvent::AccountBlockedMultiSignature, BannerState::AlwaysActive);
        let token_stake = Banner {
            asset: Some(token),
            ..Banner::mock(BannerEvent::Stake, BannerState::Active)
        };
        let stored = vec![
            Banner {
                asset: Some(Asset::from_chain(Chain::Tron)),
                ..warning.clone()
            },
            Banner::mock(BannerEvent::Stake, BannerState::Active),
            Banner::mock(BannerEvent::AccountActivation, BannerState::Active),
            Banner::mock(BannerEvent::TradePerpetuals, BannerState::Active),
            Banner::mock(BannerEvent::Onboarding, BannerState::Active),
            warning.clone(),
            token_stake.clone(),
            token_stake.clone(),
        ];
        assert_eq!(visible_banners(stored, &context), vec![warning, token_stake]);
    }

    #[test]
    fn test_asset_scope_excludes_warnings_without_an_asset_or_with_cancelled_state() {
        let stored = vec![
            Banner {
                asset: None,
                ..Banner::mock(BannerEvent::AccountBlockedMultiSignature, BannerState::AlwaysActive)
            },
            Banner::mock(BannerEvent::AccountBlockedMultiSignature, BannerState::Cancelled),
        ];
        assert!(visible_banners(stored, &GemBannerContext::mock()).is_empty());
    }

    #[test]
    fn test_visible_banners_order_and_wallet_rules() {
        let stored = vec![Banner::mock(BannerEvent::Stake, BannerState::Active), Banner::mock(BannerEvent::AccountActivation, BannerState::AlwaysActive)];
        let suspicious = GemBannerContext {
            asset_rank_score: Some(5),
            ..GemBannerContext::mock()
        };
        let banners = visible_banners(stored, &suspicious);
        assert_eq!(events(&banners), vec![BannerEvent::AccountActivation, BannerEvent::SuspiciousAsset, BannerEvent::Stake]);
        assert_eq!(banners[0].state, BannerState::AlwaysActive);
        assert_eq!(banners[2].state, BannerState::Active);

        let wallet = vec![Banner::mock(BannerEvent::Onboarding, BannerState::AlwaysActive)];
        let without_asset = GemBannerContext { asset: None, ..GemBannerContext::mock() };
        assert!(visible_banners(wallet.clone(), &without_asset).is_empty());
        let empty = GemBannerContext { is_wallet_empty: true, ..without_asset };
        assert_eq!(events(&visible_banners(wallet, &empty)), vec![BannerEvent::Onboarding]);
    }

    #[test]
    fn test_state_and_event_policies() {
        assert!(is_visible(BannerState::Active));
        assert!(is_visible(BannerState::AlwaysActive));
        assert!(!is_visible(BannerState::Cancelled));
        assert_eq!(default_state(BannerEvent::ActivateAsset), BannerState::AlwaysActive);
        assert_eq!(default_state(BannerEvent::Stake), BannerState::Active);
    }

    #[test]
    fn test_banner_content_names_the_asset_field_per_line() {
        let ethereum = Asset::from_chain(Chain::Ethereum);
        let stake = banner_content(BannerEvent::Stake, Some(&ethereum), BannerState::Active, Platform::IOS);
        assert_eq!(stake.title, Some(GemBannerTitle::Stake { asset_name: "Ethereum".to_string() }));
        assert_eq!(stake.description, Some(GemBannerDescription::Stake { asset_symbol: "ETH".to_string() }));

        let arbitrum = Asset::from_chain(Chain::Arbitrum);
        assert_eq!(arbitrum.name, "Arbitrum ETH");
        assert_eq!(
            banner_content(BannerEvent::AccountBlockedMultiSignature, Some(&arbitrum), BannerState::Active, Platform::IOS).description,
            Some(GemBannerDescription::ExternallyControlledAccount { network_name: "Arbitrum".to_string() })
        );

        let usdc = Asset::mock_ethereum_usdc();
        assert_eq!(
            banner_content(BannerEvent::ActivateAsset, Some(&usdc), BannerState::Active, Platform::IOS).description,
            Some(GemBannerDescription::ActivateAsset {
                asset_symbol: "USDC".to_string(),
                network_name: "Ethereum".to_string(),
            })
        );

        let without_asset = banner_content(BannerEvent::Stake, None, BannerState::Active, Platform::IOS);
        assert_eq!(without_asset.title, None);
        assert_eq!(without_asset.description, None);
    }

    #[test]
    fn test_banner_content_names_its_layout_its_buttons_and_whether_it_closes() {
        let ethereum = Asset::from_chain(Chain::Ethereum);
        let onboarding = banner_content(BannerEvent::Onboarding, None, BannerState::Active, Platform::IOS);
        let stake = banner_content(BannerEvent::Stake, Some(&ethereum), BannerState::Active, Platform::IOS);

        assert_eq!(onboarding.style, GemBannerStyle::Welcome);
        assert_eq!(onboarding.buttons, vec![GemBannerButton::Buy, GemBannerButton::Receive]);
        assert_eq!(stake.style, GemBannerStyle::List);
        assert!(stake.buttons.is_empty(), "a list banner is its own action");

        assert!(stake.can_close);
        assert!(
            !banner_content(BannerEvent::Stake, Some(&ethereum), BannerState::AlwaysActive, Platform::IOS).can_close,
            "a banner that is always active cannot be dismissed"
        );
    }

    #[test]
    fn test_banner_content_drops_the_activation_description_without_a_fee() {
        let xrp = Asset::from_chain(Chain::Xrp);
        assert_eq!(
            banner_content(BannerEvent::AccountActivation, Some(&xrp), BannerState::Active, Platform::IOS).description,
            Some(GemBannerDescription::AccountActivation {
                network_name: "XRP".to_string(),
                fee: GemFormattedNumber::asset_amount(&num_bigint::BigInt::from(1_000_000), &xrp, GemValueStyle::Auto),
            })
        );

        let ethereum = Asset::from_chain(Chain::Ethereum);
        assert_eq!(ethereum.id.chain.account_activation_fee(), None);
        let without_fee = banner_content(BannerEvent::AccountActivation, Some(&ethereum), BannerState::Active, Platform::IOS);
        assert_eq!(without_fee.description, None);
        assert_eq!(destination_link(&without_fee), None);
        assert_eq!(
            destination_link(&banner_content(BannerEvent::AccountActivation, Some(&xrp), BannerState::Active, Platform::IOS)),
            account_activation_fee_url(Chain::Xrp)
        );
        assert_eq!(
            destination_link(&banner_content(BannerEvent::SuspiciousAsset, Some(&ethereum), BannerState::Active, Platform::IOS)),
            Some(DocsUrl::TokenVerification.url_for(Platform::IOS)),
            "a docs link carries the app's source"
        );
        assert_eq!(without_fee.title, Some(GemBannerTitle::AccountActivation));
    }

    fn destination_kind(destination: Option<&GemBannerDestination>) -> &'static str {
        match destination {
            None => "none",
            Some(GemBannerDestination::Stake) => "stake",
            Some(GemBannerDestination::ActivateAsset { .. }) => "activate asset",
            Some(GemBannerDestination::Perpetuals) => "perpetuals",
            Some(GemBannerDestination::Url { .. }) => "url",
        }
    }

    fn destination_link(content: &GemBannerContent) -> Option<String> {
        match content.destination.as_ref()? {
            GemBannerDestination::Url { url } => Some(url.clone()),
            GemBannerDestination::Stake | GemBannerDestination::ActivateAsset { .. } | GemBannerDestination::Perpetuals => None,
        }
    }

    #[test]
    fn test_banner_destination_per_event() {
        let usdc = Asset::mock_ethereum_usdc();
        let kind = |event| destination_kind(banner_content(event, Some(&usdc), BannerState::Active, Platform::IOS).destination.as_ref());

        assert_eq!(kind(BannerEvent::Stake), "stake");
        assert_eq!(kind(BannerEvent::ActivateAsset), "activate asset");
        assert_eq!(kind(BannerEvent::TradePerpetuals), "perpetuals");
        assert_eq!(kind(BannerEvent::AccountBlockedMultiSignature), "url");
        assert_eq!(kind(BannerEvent::SuspiciousAsset), "url");
        assert_eq!(kind(BannerEvent::Onboarding), "none");

        let xrp = Asset::from_chain(Chain::Xrp);
        assert_eq!(destination_kind(banner_content(BannerEvent::AccountActivation, Some(&xrp), BannerState::Active, Platform::IOS).destination.as_ref()), "url");
        assert_eq!(destination_kind(banner_content(BannerEvent::ActivateAsset, None, BannerState::Active, Platform::IOS).destination.as_ref()), "none");
    }

    #[test]
    fn test_activate_asset_destination_carries_an_account_activation_transfer() {
        let usdc = Asset::mock_ethereum_usdc();
        let Some(GemBannerDestination::ActivateAsset { transfer }) = banner_content(BannerEvent::ActivateAsset, Some(&usdc), BannerState::Active, Platform::IOS).destination else {
            panic!("the activate asset banner must carry its transfer");
        };
        let TransactionInputType::Account {
            asset,
            account_type: AccountDataType::Activate,
        } = &transfer.input_type
        else {
            panic!("the activate asset banner must build an account activation");
        };

        assert_eq!(asset.id, usdc.id);
        assert!(transfer.recipient.address.is_empty());
        assert_eq!(transfer.value, GemBigInt::from(0));
        assert!(!transfer.use_max_amount);
    }

    #[test]
    fn test_banner_icon_per_event() {
        let stellar = Asset::from_chain(Chain::Stellar);
        let icon = |event| banner_content(event, Some(&stellar), BannerState::Active, Platform::IOS).icon;
        assert_eq!(icon(BannerEvent::Stake), Some(GemBannerIcon::MoneyBag));
        assert_eq!(icon(BannerEvent::AccountActivation), Some(GemBannerIcon::Network { chain: Chain::Stellar }));
        assert_eq!(icon(BannerEvent::ActivateAsset), Some(GemBannerIcon::Network { chain: Chain::Stellar }));
        assert_eq!(icon(BannerEvent::AccountBlockedMultiSignature), Some(GemBannerIcon::Warning));
        assert_eq!(icon(BannerEvent::SuspiciousAsset), Some(GemBannerIcon::Suspicious));
        assert_eq!(icon(BannerEvent::Onboarding), Some(GemBannerIcon::Bitcoin));
        assert_eq!(icon(BannerEvent::TradePerpetuals), Some(GemBannerIcon::Perpetuals));
        assert_eq!(banner_content(BannerEvent::AccountActivation, None, BannerState::Active, Platform::IOS).icon, None);
    }
}
