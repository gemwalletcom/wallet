use primitives::asset_constants::{HYPERCORE_PERPETUAL_USDC_ASSET_ID, HYPERCORE_SPOT_USDC_ASSET_ID};
use primitives::{AssetFiatValue, BannerEvent, Chain, PerpetualAccountMode, PerpetualBalance, WalletType};

use crate::services::banner::GemBannerRow;
use crate::services::perpetual::rules::spot_is_perpetual_collateral;

use crate::services::assets::model::{GemHeaderActions, GemHeaderButton, GemHeaderButtonAction};

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemPerpetualCollateral {
    pub balance: PerpetualBalance,
    pub price: f64,
}

pub fn shows_initial_loading(initial_load_completed: bool, assets_timestamp: u64) -> bool {
    !initial_load_completed && assets_timestamp == 0
}

pub fn wallet_balances(balances: Vec<AssetFiatValue>, collateral: Option<GemPerpetualCollateral>, mode: PerpetualAccountMode) -> Vec<AssetFiatValue> {
    let Some(collateral) = collateral else {
        return balances;
    };
    let spot_is_collateral = spot_is_perpetual_collateral(mode);
    balances
        .into_iter()
        .filter(|balance| !(spot_is_collateral && balance.asset_id == *HYPERCORE_SPOT_USDC_ASSET_ID))
        .chain([AssetFiatValue {
            asset_id: HYPERCORE_PERPETUAL_USDC_ASSET_ID.clone(),
            amount: collateral.balance.available + collateral.balance.reserved,
            price: collateral.price,
            price_change_percentage_24h: 0.0,
        }])
        .collect()
}

pub fn header_buttons_enabled(banners: &[GemBannerRow]) -> bool {
    !banners.iter().any(|row| row.banner.event == BannerEvent::AccountBlockedMultiSignature)
}

pub fn header_actions(wallet_type: WalletType, chains: &[Chain], is_enabled: bool) -> GemHeaderActions {
    match wallet_type {
        WalletType::View => GemHeaderActions::WatchOnly,
        WalletType::Multicoin | WalletType::Single | WalletType::PrivateKey => GemHeaderActions::Buttons {
            buttons: header_buttons(wallet_type, chains, is_enabled),
        },
    }
}

fn header_buttons(wallet_type: WalletType, chains: &[Chain], is_enabled: bool) -> Vec<GemHeaderButton> {
    [
        Some(GemHeaderButtonAction::Send { asset_id: None }),
        Some(GemHeaderButtonAction::Receive { asset_id: None }),
        Some(GemHeaderButtonAction::Buy { asset_id: None }),
        swaps(wallet_type, chains).then_some(GemHeaderButtonAction::Swap { pay_asset_id: None, receive_asset_id: None }),
    ]
    .into_iter()
    .flatten()
    .map(|action| GemHeaderButton::new(action, is_enabled))
    .collect()
}

fn swaps(wallet_type: WalletType, chains: &[Chain]) -> bool {
    match wallet_type {
        WalletType::Multicoin => true,
        WalletType::Single | WalletType::PrivateKey => chains.first().is_some_and(Chain::is_swap_supported),
        WalletType::View => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::assets::model::GemHeaderButtonKind;

    fn buttons(wallet_type: WalletType, chain: Chain, is_enabled: bool) -> Vec<GemHeaderButton> {
        match header_actions(wallet_type, &[chain], is_enabled) {
            GemHeaderActions::Buttons { buttons } => buttons,
            GemHeaderActions::WatchOnly => panic!("the header offers no buttons"),
        }
    }

    fn kinds(wallet_type: WalletType, chain: Chain) -> Vec<GemHeaderButtonKind> {
        buttons(wallet_type, chain, true).into_iter().map(|button| button.kind).collect()
    }

    #[test]
    fn test_wallet_balances_count_perpetual_collateral_at_par_without_a_day_change() {
        let eth = AssetFiatValue::mock(2.0, 10.0, 5.0);
        let collateral = GemPerpetualCollateral {
            balance: PerpetualBalance {
                available: 30.0,
                reserved: 20.0,
                withdrawable: 25.0,
            },
            price: 0.92,
        };

        assert_eq!(wallet_balances(vec![eth.clone()], None, PerpetualAccountMode::Standard), vec![eth.clone()]);
        assert_eq!(
            wallet_balances(vec![eth.clone()], Some(collateral), PerpetualAccountMode::Standard),
            vec![
                eth,
                AssetFiatValue {
                    asset_id: HYPERCORE_PERPETUAL_USDC_ASSET_ID.clone(),
                    ..AssetFiatValue::mock(50.0, 0.92, 0.0)
                }
            ],
            "collateral is what is available plus what positions hold, priced like every other entry"
        );
    }

    #[test]
    fn test_wallet_balances_count_a_unified_account_spot_usdc_once() {
        let spot_usdc = AssetFiatValue {
            asset_id: HYPERCORE_SPOT_USDC_ASSET_ID.clone(),
            ..AssetFiatValue::mock(30.0, 1.0, 0.0)
        };
        let collateral = || GemPerpetualCollateral {
            balance: PerpetualBalance {
                available: 30.0,
                reserved: 20.0,
                withdrawable: 30.0,
            },
            price: 1.0,
        };
        let total = |balances: Vec<AssetFiatValue>| balances.iter().map(|balance| balance.amount * balance.price).sum::<f64>();

        assert_eq!(
            total(wallet_balances(vec![spot_usdc.clone()], Some(collateral()), PerpetualAccountMode::Standard)),
            80.0,
            "a standard account's spot USDC is its own money"
        );
        assert_eq!(
            total(wallet_balances(vec![spot_usdc.clone()], Some(collateral()), PerpetualAccountMode::Unified)),
            50.0,
            "a unified account's spot USDC is the perpetual balance, margin included"
        );
        assert_eq!(total(wallet_balances(vec![spot_usdc], None, PerpetualAccountMode::Unified)), 30.0, "without a perpetual balance the spot USDC still counts");
    }

    #[test]
    fn test_shows_initial_loading_only_before_the_first_discovery() {
        assert!(shows_initial_loading(false, 0));
        assert!(!shows_initial_loading(true, 0));
        assert!(!shows_initial_loading(false, 1));
        assert!(!shows_initial_loading(true, 1));
    }

    #[test]
    fn test_a_blocked_multi_signature_account_disables_the_header_buttons() {
        use primitives::{Banner, BannerState};
        let row = |event| GemBannerRow::new(Banner::mock(event, BannerState::Active), primitives::Platform::IOS);

        assert!(header_buttons_enabled(&[]));
        assert!(header_buttons_enabled(&[row(BannerEvent::Onboarding)]));
        assert!(!header_buttons_enabled(&[row(BannerEvent::Onboarding), row(BannerEvent::AccountBlockedMultiSignature)]));
    }

    #[test]
    fn test_the_header_offers_swap_to_multicoin_and_swappable_single_chain_wallets_only() {
        use GemHeaderButtonKind::*;
        assert_eq!(kinds(WalletType::Multicoin, Chain::Bitcoin), vec![Send, Receive, Buy, Swap]);
        assert!(
            buttons(WalletType::Multicoin, Chain::Bitcoin, true).iter().all(|button| match &button.action {
                GemHeaderButtonAction::Send { asset_id } | GemHeaderButtonAction::Receive { asset_id } | GemHeaderButtonAction::Buy { asset_id } => asset_id.is_none(),
                GemHeaderButtonAction::Swap { pay_asset_id, receive_asset_id } => pay_asset_id.is_none() && receive_asset_id.is_none(),
                other => panic!("the wallet header offers no {other:?}"),
            }),
            "the wallet header asks for the asset after the action"
        );
        assert_eq!(kinds(WalletType::Single, Chain::Ethereum), vec![Send, Receive, Buy, Swap]);
        assert_eq!(kinds(WalletType::PrivateKey, Chain::Solana), vec![Send, Receive, Buy, Swap]);
        assert_eq!(kinds(WalletType::Single, Chain::Mayachain), vec![Send, Receive, Buy]);
        assert_eq!(header_actions(WalletType::View, &[Chain::Ethereum], true), GemHeaderActions::WatchOnly);
        assert!(buttons(WalletType::Multicoin, Chain::Ethereum, false).iter().all(|button| !button.is_enabled));
    }
}
