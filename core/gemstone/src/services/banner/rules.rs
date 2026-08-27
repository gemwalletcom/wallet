use primitives::{AssetId, BannerEvent, BannerState, Chain, Wallet, WalletId, WalletSource};

use super::model::{GemBannerAction, GemBannerContext, GemBannerItem, GemBannerKey};

const ACCOUNT_ACTIVATION_CHAINS: [Chain; 3] = [Chain::Xrp, Chain::Stellar, Chain::Algorand];
const TRADE_PERPETUALS_CHAINS: [Chain; 2] = [Chain::HyperCore, Chain::Hyperliquid];

const ENABLE_NOTIFICATIONS_MINIMUM_LAUNCHES: u32 = 3;
const SUSPICIOUS_RANK_SCORE: i32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BannerClose {
    Close,
    Keep,
    AfterPermission,
}

pub fn close_decision(action: &GemBannerAction) -> BannerClose {
    match action {
        GemBannerAction::Close => BannerClose::Close,
        GemBannerAction::Button => BannerClose::Keep,
        GemBannerAction::Event { event } => {
            if closes_on_action(*event) {
                BannerClose::AfterPermission
            } else {
                BannerClose::Keep
            }
        }
    }
}

pub fn event_key(wallet_id: Option<WalletId>, asset_id: Option<AssetId>, event: BannerEvent) -> GemBannerKey {
    GemBannerKey {
        wallet_id,
        asset_id,
        chain: None,
        event,
    }
}

pub fn is_visible(state: BannerState) -> bool {
    match state {
        BannerState::Active | BannerState::AlwaysActive => true,
        BannerState::Cancelled => false,
    }
}

pub fn default_state(event: BannerEvent) -> BannerState {
    match event {
        BannerEvent::ActivateAsset | BannerEvent::AccountBlockedMultiSignature => BannerState::AlwaysActive,
        BannerEvent::Stake
        | BannerEvent::AccountActivation
        | BannerEvent::EnableNotifications
        | BannerEvent::SuspiciousAsset
        | BannerEvent::Onboarding
        | BannerEvent::TradePerpetuals => BannerState::Active,
    }
}

pub fn closes_on_action(event: BannerEvent) -> bool {
    match event {
        BannerEvent::EnableNotifications => true,
        BannerEvent::Stake
        | BannerEvent::AccountActivation
        | BannerEvent::AccountBlockedMultiSignature
        | BannerEvent::ActivateAsset
        | BannerEvent::SuspiciousAsset
        | BannerEvent::Onboarding
        | BannerEvent::TradePerpetuals => false,
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

pub fn is_available(event: BannerEvent, context: &GemBannerContext) -> bool {
    match event {
        BannerEvent::EnableNotifications => context.notifications_available && context.launch_count >= ENABLE_NOTIFICATIONS_MINIMUM_LAUNCHES,
        _ => true,
    }
}

pub fn is_visible_event(event: BannerEvent, context: &GemBannerContext) -> bool {
    match event {
        BannerEvent::EnableNotifications | BannerEvent::AccountBlockedMultiSignature => true,
        BannerEvent::AccountActivation => !context.has_asset || !context.has_available_balance,
        BannerEvent::Stake => context.has_asset && !context.has_stake_balance,
        BannerEvent::ActivateAsset => context.has_asset && !context.is_asset_activated,
        BannerEvent::SuspiciousAsset => context.has_asset && is_suspicious(context),
        BannerEvent::TradePerpetuals => context.has_asset && context.has_perpetuals_support,
        BannerEvent::Onboarding => !context.has_asset && context.is_wallet_empty,
    }
}

pub fn visible_banners(stored: Vec<GemBannerItem>, context: &GemBannerContext) -> Vec<GemBannerItem> {
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
    context.asset_rank_score.is_some_and(|score| score <= SUSPICIOUS_RANK_SCORE)
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
        BannerEvent::EnableNotifications => 5,
        BannerEvent::Stake => 6,
        BannerEvent::TradePerpetuals => 7,
    }
}

pub fn suggested_events(context: &GemBannerContext) -> Vec<BannerEvent> {
    let mut events = Vec::new();
    if !context.has_wallet && !context.has_asset {
        events.push(BannerEvent::EnableNotifications);
    }
    if context.has_asset {
        if context.is_stakeable && !context.has_stake_balance {
            events.push(BannerEvent::Stake);
        }
        if !context.is_asset_activated {
            events.push(BannerEvent::ActivateAsset);
        }
    }
    events.into_iter().filter(|event| is_available(*event, context)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{WalletId, WalletType};

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
            has_wallet: true,
            has_asset,
            is_stakeable: true,
            has_stake_balance: false,
            has_available_balance: false,
            is_asset_activated: true,
            asset_rank_score: Some(50),
            has_perpetuals_support: true,
            is_wallet_empty: false,
            notifications_available: true,
            launch_count: 3,
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
            has_perpetuals_support: false,
            ..context(true)
        };
        assert!(visible_banners(perpetuals, &unsupported).is_empty());
    }

    #[test]
    fn test_visible_banners_order_and_wallet_rules() {
        let stored = vec![
            item(BannerEvent::Stake, BannerState::Active),
            item(BannerEvent::EnableNotifications, BannerState::Cancelled),
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

        let wallet = vec![
            item(BannerEvent::Onboarding, BannerState::AlwaysActive),
            item(BannerEvent::EnableNotifications, BannerState::Active),
        ];
        assert_eq!(events(&visible_banners(wallet.clone(), &context(false))), vec![BannerEvent::EnableNotifications]);
        let empty = GemBannerContext {
            is_wallet_empty: true,
            ..context(false)
        };
        assert_eq!(events(&visible_banners(wallet, &empty)), vec![BannerEvent::Onboarding, BannerEvent::EnableNotifications]);
    }

    #[test]
    fn test_state_and_event_policies() {
        assert!(is_visible(BannerState::Active));
        assert!(is_visible(BannerState::AlwaysActive));
        assert!(!is_visible(BannerState::Cancelled));
        assert_eq!(default_state(BannerEvent::ActivateAsset), BannerState::AlwaysActive);
        assert_eq!(default_state(BannerEvent::Stake), BannerState::Active);
        assert!(closes_on_action(BannerEvent::EnableNotifications));
        assert!(!closes_on_action(BannerEvent::Stake));
    }

    #[test]
    fn test_notifications_banner_waits_for_launches() {
        let mut no_asset = context(false);
        no_asset.has_wallet = false;
        assert!(is_available(BannerEvent::EnableNotifications, &no_asset));
        assert_eq!(suggested_events(&no_asset), vec![BannerEvent::EnableNotifications]);

        no_asset.launch_count = ENABLE_NOTIFICATIONS_MINIMUM_LAUNCHES - 1;
        assert!(!is_available(BannerEvent::EnableNotifications, &no_asset));
        assert!(suggested_events(&no_asset).is_empty());
        assert!(is_available(BannerEvent::Stake, &no_asset));

        let mut asset = context(true);
        asset.is_asset_activated = false;
        assert_eq!(suggested_events(&asset), vec![BannerEvent::Stake, BannerEvent::ActivateAsset]);
    }

    #[test]
    fn test_close_decision_asks_for_permission_only_on_closing_events() {
        assert_eq!(close_decision(&GemBannerAction::Close), BannerClose::Close);
        assert_eq!(close_decision(&GemBannerAction::Button), BannerClose::Keep);
        assert_eq!(
            close_decision(&GemBannerAction::Event {
                event: BannerEvent::EnableNotifications
            }),
            BannerClose::AfterPermission
        );
        assert_eq!(
            close_decision(&GemBannerAction::Event {
                event: BannerEvent::AccountActivation
            }),
            BannerClose::Keep
        );
    }

    #[test]
    fn test_event_key_targets_the_wallet_and_asset_without_a_chain() {
        let wallet_id = WalletId::Multicoin("wallet".into());
        let asset_id = AssetId::from_chain(Chain::Ethereum);

        let key = event_key(Some(wallet_id.clone()), Some(asset_id.clone()), BannerEvent::AccountActivation);

        assert_eq!(key.wallet_id, Some(wallet_id));
        assert_eq!(key.asset_id, Some(asset_id));
        assert_eq!(key.chain, None);
        assert_eq!(key.event, BannerEvent::AccountActivation);
    }
}
