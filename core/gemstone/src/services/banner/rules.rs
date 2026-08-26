use primitives::{AssetId, BannerEvent, BannerState, Chain, Wallet, WalletSource};

use super::model::{GemBannerContext, GemBannerKey};

const ACCOUNT_ACTIVATION_CHAINS: [Chain; 3] = [Chain::Xrp, Chain::Stellar, Chain::Algorand];
const TRADE_PERPETUALS_CHAINS: [Chain; 2] = [Chain::HyperCore, Chain::Hyperliquid];

const ENABLE_NOTIFICATIONS_MINIMUM_LAUNCHES: u32 = 3;

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
        chain: None,
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
            chain: None,
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
        assert!(keys.iter().all(|key| key.wallet_id.is_none() && key.chain.is_none()));
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
}
