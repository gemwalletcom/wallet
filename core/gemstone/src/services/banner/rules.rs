use primitives::{BannerEvent, BannerState};

use super::model::GemBannerContext;

const ENABLE_NOTIFICATIONS_MINIMUM_LAUNCHES: u32 = 3;

pub fn is_visible(state: BannerState) -> bool {
    match state {
        BannerState::Active | BannerState::AlwaysActive => true,
        BannerState::Cancelled => false,
    }
}

pub fn default_state(event: BannerEvent) -> BannerState {
    match event {
        BannerEvent::ActivateAsset => BannerState::AlwaysActive,
        BannerEvent::Stake
        | BannerEvent::AccountActivation
        | BannerEvent::EnableNotifications
        | BannerEvent::AccountBlockedMultiSignature
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
