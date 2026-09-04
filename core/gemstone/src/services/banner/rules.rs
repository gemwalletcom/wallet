use crate::models::custom_types::GemBigInt;
use primitives::{Asset, AssetId, BannerEvent, BannerState, Chain, ChainAsset, VerificationStatus, Wallet, WalletSource};

use super::model::{GemBannerAmount, GemBannerContent, GemBannerContext, GemBannerDescription, GemBannerIcon, GemBannerItem, GemBannerKey, GemBannerLink, GemBannerTitle};
use crate::config::chain::account_activation_fee_url;
use crate::config::docs::DocsUrl;

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
    ACCOUNT_ACTIVATION_CHAINS
        .into_iter()
        .map(|chain| asset_key(chain, BannerEvent::AccountActivation))
        .chain(onboarding)
        .collect()
}

fn is_visible_event(event: BannerEvent, context: &GemBannerContext) -> bool {
    match event {
        BannerEvent::AccountBlockedMultiSignature => true,
        BannerEvent::AccountActivation => !context.has_asset || !context.has_available_balance,
        BannerEvent::Stake => context.has_asset && !context.has_stake_balance,
        BannerEvent::ActivateAsset => context.has_asset && !context.is_asset_activated,
        BannerEvent::SuspiciousAsset => context.has_asset && is_suspicious(context),
        BannerEvent::TradePerpetuals => context.has_asset && context.wallet.as_ref().is_some_and(crate::services::perpetual::rules::supports_perpetuals),
        BannerEvent::Onboarding => !context.has_asset && context.is_wallet_empty,
    }
}

pub fn banner_content(event: BannerEvent, asset: Option<&Asset>) -> GemBannerContent {
    GemBannerContent {
        icon: banner_icon(event, asset.map(|asset| asset.id.chain)),
        title: banner_title(event, asset),
        description: banner_description(event, asset),
        link: banner_link(event, asset.map(|asset| asset.id.chain)),
    }
}

fn banner_link(event: BannerEvent, chain: Option<Chain>) -> Option<GemBannerLink> {
    match event {
        BannerEvent::Stake | BannerEvent::ActivateAsset | BannerEvent::Onboarding | BannerEvent::TradePerpetuals => None,
        BannerEvent::AccountActivation => account_activation_fee_url(chain?).map(|url| GemBannerLink::External { url }),
        BannerEvent::AccountBlockedMultiSignature => Some(GemBannerLink::Docs {
            item: DocsUrl::TronMultiSignature,
        }),
        BannerEvent::SuspiciousAsset => Some(GemBannerLink::Docs { item: DocsUrl::TokenVerification }),
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
        BannerEvent::Stake => Some(GemBannerDescription::Stake {
            asset_symbol: asset?.symbol.clone(),
        }),
        BannerEvent::AccountActivation => {
            let asset = asset?;
            Some(GemBannerDescription::AccountActivation {
                network_name: network_name(asset.id.chain),
                fee: GemBannerAmount {
                    value: GemBigInt::from(asset.id.chain.account_activation_fee()?),
                    decimals: asset.decimals,
                    symbol: asset.symbol.clone(),
                },
            })
        }
        BannerEvent::AccountBlockedMultiSignature => Some(GemBannerDescription::MultiSignatureBlocked {
            network_name: network_name(asset?.id.chain),
        }),
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

pub(super) fn visible_banners(stored: Vec<GemBannerItem>, context: &GemBannerContext) -> Vec<GemBannerItem> {
    let mut banners: Vec<GemBannerItem> = Vec::new();
    for item in stored.into_iter().chain(extra_banners()) {
        if banners.iter().any(|existing| existing.event == item.event) {
            continue;
        }
        if is_visible(item.state) && is_visible_event(item.event, context) {
            banners.push(item);
        }
    }
    banners.sort_by_key(|item| (state_priority(item.state), event_priority(item.event)));
    banners
}

fn extra_banners() -> Vec<GemBannerItem> {
    [BannerEvent::ActivateAsset, BannerEvent::SuspiciousAsset]
        .into_iter()
        .map(|event| GemBannerItem {
            event,
            state: default_state(event),
        })
        .collect()
}

fn is_suspicious(context: &GemBannerContext) -> bool {
    context
        .asset_rank_score
        .is_some_and(|score| VerificationStatus::from_rank(score) == VerificationStatus::Suspicious)
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
    use super::*;
    use primitives::{Account, WalletId, WalletType};

    #[test]
    fn test_setup_keys() {
        let keys = setup_keys();
        assert!(keys.iter().all(|key| key.wallet_id.is_none()));
        assert!(
            keys.iter()
                .any(|key| key.event == BannerEvent::Stake && key.asset_id == Some(AssetId::from_chain(Chain::Cosmos)))
        );
        assert!(
            !keys
                .iter()
                .any(|key| key.event == BannerEvent::Stake && key.asset_id == Some(AssetId::from_chain(Chain::Bitcoin)))
        );
        assert_eq!(keys.iter().filter(|key| key.event == BannerEvent::TradePerpetuals).count(), 2);
    }

    #[test]
    fn test_wallet_setup_keys() {
        let mut wallet = Wallet {
            id: WalletId::Multicoin("0x1".to_string()),
            external_id: None,
            name: "wallet".to_string(),
            index: 0,
            wallet_type: WalletType::Multicoin,
            accounts: vec![],
            is_pinned: false,
            image_url: None,
            source: WalletSource::Import,
        };
        let imported = wallet_setup_keys(&wallet);
        assert_eq!(imported.len(), 3);
        assert!(imported.iter().all(|key| key.event == BannerEvent::AccountActivation));

        wallet.source = WalletSource::Create;
        let created = wallet_setup_keys(&wallet);
        assert_eq!(created.len(), 4);
        assert_eq!(
            created.last().map(|key| (key.event, key.wallet_id.clone())),
            Some((BannerEvent::Onboarding, Some(wallet.id)))
        );
    }

    fn context(has_asset: bool) -> GemBannerContext {
        GemBannerContext {
            wallet: Some(Wallet::mock_with_accounts(Account::mock_chains(&[Chain::Ethereum, Chain::HyperCore], "address"))),
            has_asset,
            is_stakeable: true,
            has_stake_balance: false,
            has_available_balance: false,
            is_asset_activated: true,
            asset_rank_score: Some(50),
            is_wallet_empty: false,
        }
    }

    fn item(event: BannerEvent, state: BannerState) -> GemBannerItem {
        GemBannerItem { event, state }
    }

    fn events(banners: &[GemBannerItem]) -> Vec<BannerEvent> {
        banners.iter().map(|banner| banner.event).collect()
    }

    #[test]
    fn test_visible_banners_asset_rules() {
        let stake = vec![item(BannerEvent::Stake, BannerState::Active)];
        assert_eq!(events(&visible_banners(stake.clone(), &context(true))), vec![BannerEvent::Stake]);
        let staked = GemBannerContext {
            has_stake_balance: true,
            ..context(true)
        };
        assert!(visible_banners(stake, &staked).is_empty());

        let inactive = GemBannerContext {
            is_asset_activated: false,
            ..context(true)
        };
        assert_eq!(events(&visible_banners(vec![], &inactive)), vec![BannerEvent::ActivateAsset]);
        assert!(visible_banners(vec![], &context(true)).is_empty());

        let suspicious = GemBannerContext {
            asset_rank_score: Some(5),
            ..context(true)
        };
        assert_eq!(events(&visible_banners(vec![], &suspicious)), vec![BannerEvent::SuspiciousAsset]);

        let activation = vec![item(BannerEvent::AccountActivation, BannerState::AlwaysActive)];
        assert_eq!(events(&visible_banners(activation.clone(), &context(true))), vec![BannerEvent::AccountActivation]);
        let funded = GemBannerContext {
            has_available_balance: true,
            ..context(true)
        };
        assert!(visible_banners(activation, &funded).is_empty());

        let perpetuals = vec![item(BannerEvent::TradePerpetuals, BannerState::Active)];
        assert_eq!(events(&visible_banners(perpetuals.clone(), &context(true))), vec![BannerEvent::TradePerpetuals]);
        let unsupported = GemBannerContext {
            wallet: Some(Wallet::mock_with_accounts(Account::mock_chains(&[Chain::Ethereum], "address"))),
            ..context(true)
        };
        assert!(visible_banners(perpetuals.clone(), &unsupported).is_empty());
        let no_wallet = GemBannerContext { wallet: None, ..context(true) };
        assert!(visible_banners(perpetuals, &no_wallet).is_empty());
    }

    #[test]
    fn test_visible_banners_order_and_wallet_rules() {
        let stored = vec![
            item(BannerEvent::Stake, BannerState::Active),
            item(BannerEvent::AccountActivation, BannerState::AlwaysActive),
        ];
        let suspicious = GemBannerContext {
            asset_rank_score: Some(5),
            ..context(true)
        };
        let banners = visible_banners(stored, &suspicious);
        assert_eq!(events(&banners), vec![BannerEvent::AccountActivation, BannerEvent::SuspiciousAsset, BannerEvent::Stake]);
        assert_eq!(banners[0].state, BannerState::AlwaysActive);
        assert_eq!(banners[2].state, BannerState::Active);

        let wallet = vec![item(BannerEvent::Onboarding, BannerState::AlwaysActive)];
        assert!(visible_banners(wallet.clone(), &context(false)).is_empty());
        let empty = GemBannerContext {
            is_wallet_empty: true,
            ..context(false)
        };
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
        let stake = banner_content(BannerEvent::Stake, Some(&ethereum));
        assert_eq!(
            stake.title,
            Some(GemBannerTitle::Stake {
                asset_name: "Ethereum".to_string()
            })
        );
        assert_eq!(stake.description, Some(GemBannerDescription::Stake { asset_symbol: "ETH".to_string() }));

        let arbitrum = Asset::from_chain(Chain::Arbitrum);
        assert_eq!(arbitrum.name, "Arbitrum ETH");
        assert_eq!(
            banner_content(BannerEvent::AccountBlockedMultiSignature, Some(&arbitrum)).description,
            Some(GemBannerDescription::MultiSignatureBlocked {
                network_name: "Arbitrum".to_string()
            })
        );

        let usdc = Asset::mock_ethereum_usdc();
        assert_eq!(
            banner_content(BannerEvent::ActivateAsset, Some(&usdc)).description,
            Some(GemBannerDescription::ActivateAsset {
                asset_symbol: "USDC".to_string(),
                network_name: "Ethereum".to_string(),
            })
        );

        let without_asset = banner_content(BannerEvent::Stake, None);
        assert_eq!(without_asset.title, None);
        assert_eq!(without_asset.description, None);
    }

    #[test]
    fn test_banner_content_drops_the_activation_description_without_a_fee() {
        let xrp = Asset::from_chain(Chain::Xrp);
        assert_eq!(
            banner_content(BannerEvent::AccountActivation, Some(&xrp)).description,
            Some(GemBannerDescription::AccountActivation {
                network_name: "XRP".to_string(),
                fee: GemBannerAmount {
                    value: GemBigInt::from(1_000_000),
                    decimals: 6,
                    symbol: "XRP".to_string(),
                },
            })
        );

        let ethereum = Asset::from_chain(Chain::Ethereum);
        assert_eq!(ethereum.id.chain.account_activation_fee(), None);
        let without_fee = banner_content(BannerEvent::AccountActivation, Some(&ethereum));
        assert_eq!(without_fee.description, None);
        assert_eq!(without_fee.link, None);
        assert_eq!(
            banner_content(BannerEvent::AccountActivation, Some(&xrp)).link,
            Some(GemBannerLink::External {
                url: account_activation_fee_url(Chain::Xrp).unwrap()
            })
        );
        assert_eq!(
            banner_content(BannerEvent::SuspiciousAsset, Some(&ethereum)).link,
            Some(GemBannerLink::Docs { item: DocsUrl::TokenVerification })
        );
        assert_eq!(without_fee.title, Some(GemBannerTitle::AccountActivation));
    }

    #[test]
    fn test_banner_icon_per_event() {
        let stellar = Asset::from_chain(Chain::Stellar);
        let icon = |event| banner_content(event, Some(&stellar)).icon;
        assert_eq!(icon(BannerEvent::Stake), Some(GemBannerIcon::MoneyBag));
        assert_eq!(icon(BannerEvent::AccountActivation), Some(GemBannerIcon::Network { chain: Chain::Stellar }));
        assert_eq!(icon(BannerEvent::ActivateAsset), Some(GemBannerIcon::Network { chain: Chain::Stellar }));
        assert_eq!(icon(BannerEvent::AccountBlockedMultiSignature), Some(GemBannerIcon::Warning));
        assert_eq!(icon(BannerEvent::SuspiciousAsset), Some(GemBannerIcon::Suspicious));
        assert_eq!(icon(BannerEvent::Onboarding), Some(GemBannerIcon::Bitcoin));
        assert_eq!(icon(BannerEvent::TradePerpetuals), Some(GemBannerIcon::Perpetuals));
        assert_eq!(banner_content(BannerEvent::AccountActivation, None).icon, None);
    }
}
