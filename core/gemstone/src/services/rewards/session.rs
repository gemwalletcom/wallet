use chrono::{DateTime, Utc};
use primitives::{Rewards, Wallet, WalletId};

use super::model::{GemRewardsResult, GemRewardsViewState, GemRewardsWallet};
use super::rules;
use crate::models::state::{GemLoad, GemLoadState};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRewardsSession {
    pub wallet: Option<GemRewardsWallet>,
    pub code: Option<String>,
    pub state: GemLoadState,
    pub rewards: Option<Rewards>,
    pub is_refreshing: bool,
}

#[uniffi::export]
impl GemRewardsSession {
    pub fn on_wallets(&self, wallets: Vec<Wallet>, current: Option<WalletId>) -> Self {
        match rules::wallet(wallets, self.wallet_id().as_ref(), current.as_ref()) {
            Some(wallet) if self.wallet_id().as_ref() == Some(&wallet.id) => Self { wallet: Some(wallet), ..self.clone() },
            Some(wallet) => self.selected(wallet),
            None => Self {
                wallet: None,
                state: GemLoadState::NoData,
                rewards: None,
                is_refreshing: false,
                ..self.clone()
            },
        }
    }

    pub fn on_select_wallet(&self, wallet_id: WalletId) -> Self {
        match self.wallet.as_ref().and_then(|wallet| wallet.selecting(&wallet_id)) {
            Some(selected) if self.wallet_id().as_ref() != Some(&selected.id) => self.selected(selected),
            _ => self.clone(),
        }
    }

    pub fn on_refreshing(&self) -> Self {
        Self {
            is_refreshing: self.wallet.is_some(),
            ..self.clone()
        }
    }

    pub fn on_result(&self, result: GemRewardsResult) -> Self {
        if self.wallet_id().as_ref() != Some(&result.wallet_id) {
            return self.clone();
        }
        let shown = self.shown().data(result.state.into_result(result.rewards));
        Self {
            state: shown.state,
            rewards: shown.value,
            is_refreshing: false,
            ..self.clone()
        }
    }

    pub fn on_rewards(&self, wallet_id: WalletId, rewards: Rewards) -> Self {
        if self.wallet_id() != Some(wallet_id) {
            return self.clone();
        }
        Self {
            state: GemLoadState::Data,
            rewards: Some(rewards),
            is_refreshing: false,
            ..self.clone()
        }
    }

    pub fn on_code_handled(&self) -> Self {
        Self { code: None, ..self.clone() }
    }

    pub fn needs_load(&self) -> bool {
        self.wallet.is_some() && (self.state == GemLoadState::Loading || self.is_refreshing)
    }

    pub fn view_state(&self, now: DateTime<Utc>) -> GemRewardsViewState {
        let is_settled = match self.state {
            GemLoadState::Data | GemLoadState::Error { .. } => !self.is_refreshing,
            GemLoadState::NoData | GemLoadState::Loading => false,
        };
        GemRewardsViewState {
            state: self.state.clone(),
            rewards: rules::state(self.rewards.as_ref(), now),
            is_refreshing: self.is_refreshing,
            wallet: self.wallet.clone(),
            incoming_code: is_settled.then(|| rules::incoming_code(self.code.as_deref(), self.wallet.as_ref())).flatten(),
        }
    }
}

impl GemRewardsSession {
    fn wallet_id(&self) -> Option<WalletId> {
        self.wallet.as_ref().map(|wallet| wallet.id.clone())
    }

    fn selected(&self, wallet: GemRewardsWallet) -> Self {
        Self {
            wallet: Some(wallet),
            state: GemLoadState::Loading,
            rewards: None,
            is_refreshing: false,
            ..self.clone()
        }
    }

    fn shown(&self) -> GemLoad<Option<Rewards>> {
        GemLoad {
            state: self.state.clone(),
            value: self.rewards.clone(),
        }
    }
}

#[uniffi::export]
pub fn rewards_session(code: Option<String>) -> GemRewardsSession {
    GemRewardsSession {
        wallet: None,
        code,
        state: GemLoadState::Loading,
        rewards: None,
        is_refreshing: false,
    }
}

#[cfg(test)]
mod tests {
    use primitives::{Chain, RewardStatus};

    use super::*;
    use crate::services::error::GemServiceError;
    use crate::services::rewards::model::GemIncomingCode;

    fn now() -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000, 0).unwrap()
    }

    fn id(value: &str) -> WalletId {
        WalletId::Multicoin(value.to_string())
    }

    fn wallets(ids: &[&str]) -> Vec<Wallet> {
        ids.iter().map(|value| Wallet::mock_with_id(id(value), &[Chain::Ethereum])).collect()
    }

    fn loaded(wallet_id: &str, state: GemLoadState, rewards: Option<Rewards>) -> GemRewardsResult {
        GemRewardsResult { wallet_id: id(wallet_id), state, rewards }
    }

    fn offline() -> GemLoadState {
        GemLoadState::Error {
            error: GemServiceError::Gateway { msg: "offline".to_string() },
        }
    }

    fn invited() -> Rewards {
        Rewards {
            code: Some("GEM123".to_string()),
            ..Rewards::mock(None, RewardStatus::Verified)
        }
    }

    fn opened(ids: &[&str], code: Option<&str>) -> GemRewardsSession {
        rewards_session(code.map(str::to_string))
            .on_wallets(wallets(ids), Some(id("0x1")))
            .on_result(loaded("0x1", GemLoadState::Data, Some(invited())))
    }

    #[test]
    fn test_a_failed_load_reads_as_a_failure_instead_of_a_wallet_without_a_code() {
        let unreachable = rewards_session(None).on_wallets(wallets(&["0x1"]), None).on_result(loaded("0x1", offline(), None));

        assert!(matches!(unreachable.view_state(now()).state, GemLoadState::Error { .. }), "a wallet with a code must not be offered the create-code screen");
        let rewards = unreachable.view_state(now()).rewards;
        assert!(
            rewards.invite_action.is_none() && !rewards.can_use_referral_code && rewards.pending_referral.is_none(),
            "a failed wallet is offered nothing to do"
        );
    }

    #[test]
    fn test_a_failed_refresh_keeps_the_code_already_on_screen() {
        let shown = opened(&["0x1"], None);

        let kept = shown.on_refreshing().on_result(loaded("0x1", offline(), None));

        assert_eq!(kept.view_state(now()).state, GemLoadState::Data);
        assert_eq!(kept.view_state(now()).rewards, shown.view_state(now()).rewards);
    }

    #[test]
    fn test_a_result_for_a_wallet_that_is_no_longer_shown_is_dropped() {
        let shown = opened(&["0x1", "0x2"], None).on_select_wallet(id("0x2"));

        assert_eq!(shown.on_result(loaded("0x1", GemLoadState::Data, Some(invited()))), shown, "the wallet moved on before the answer arrived");
    }

    #[test]
    fn test_selecting_another_wallet_starts_over_and_reselecting_the_same_one_does_not() {
        let shown = opened(&["0x1", "0x2"], None);

        assert_eq!(shown.on_select_wallet(id("0x1")), shown);
        assert_eq!(shown.on_select_wallet(id("0x3")), shown, "a wallet the screen does not offer cannot be chosen");
        let switched = shown.on_select_wallet(id("0x2"));
        assert_eq!(switched.view_state(now()).state, GemLoadState::Loading);
        assert_eq!(switched.view_state(now()).wallet.map(|wallet| (wallet.id, wallet.row.id)), Some((id("0x2"), id("0x2"))));
    }

    #[test]
    fn test_a_fresh_wallet_list_keeps_the_chosen_wallet_and_its_rewards_without_a_reload() {
        let chosen = opened(&["0x1", "0x2"], None).on_select_wallet(id("0x2")).on_result(loaded("0x2", GemLoadState::Data, Some(invited())));
        let mut renamed = wallets(&["0x1", "0x2"]);
        renamed[1].name = "Renamed".to_string();

        let updated = chosen.on_wallets(renamed, Some(id("0x1")));

        assert_eq!(
            updated.wallet.as_ref().map(|wallet| (wallet.id.clone(), wallet.row.name.clone())),
            Some((id("0x2"), "Renamed".to_string())),
            "the user's choice outlives the current wallet"
        );
        assert_eq!(updated.rewards, chosen.rewards);
        assert!(!updated.needs_load());
    }

    #[test]
    fn test_a_wallet_list_without_the_chosen_wallet_moves_to_the_current_one() {
        let shown = opened(&["0x1", "0x2"], None);

        let moved = shown.on_wallets(wallets(&["0x2", "0x3"]), Some(id("0x3")));

        assert_eq!(moved.wallet.map(|wallet| wallet.id), Some(id("0x3")));
        assert_eq!(moved.state, GemLoadState::Loading);
        assert_eq!(moved.rewards, None, "the gone wallet's rewards must not show under the other one");
    }

    #[test]
    fn test_a_load_is_needed_once_a_wallet_is_known_and_again_on_refresh() {
        let session = rewards_session(None);
        assert!(!session.needs_load(), "no wallet to load yet");

        let known = session.on_wallets(wallets(&["0x1"]), None);
        assert!(known.needs_load());

        let shown = known.on_result(loaded("0x1", GemLoadState::Data, Some(invited())));
        assert!(!shown.needs_load());
        assert!(shown.on_refreshing().needs_load());
        assert!(!shown.on_refreshing().on_result(loaded("0x1", GemLoadState::Data, None)).view_state(now()).is_refreshing);
    }

    #[test]
    fn test_no_multicoin_wallet_reads_as_no_data_offers_no_code_and_has_nothing_to_refresh() {
        let mut single = wallets(&["0x1"]);
        single[0].wallet_type = primitives::WalletType::PrivateKey;

        let session = rewards_session(Some("friend".to_string())).on_wallets(single, None);

        let view = session.view_state(now());
        assert_eq!(view.state, GemLoadState::NoData);
        assert_eq!(view.wallet, None);
        assert_eq!(view.incoming_code, None);
        assert!(!session.needs_load());
        assert!(!session.on_refreshing().view_state(now()).is_refreshing, "a pull with no wallet must not spin forever");
    }

    #[test]
    fn test_an_incoming_code_waits_for_the_first_load_to_settle_and_is_offered_until_handled() {
        let known = rewards_session(Some(" friend ".to_string())).on_wallets(wallets(&["0x1"]), None);
        assert_eq!(known.view_state(now()).incoming_code, None, "nothing to activate while the first load runs");

        let activate = Some(GemIncomingCode::Activate { code: "friend".to_string() });
        let single = known.on_result(loaded("0x1", GemLoadState::Data, Some(invited())));
        assert_eq!(single.view_state(now()).incoming_code, activate);
        assert_eq!(known.on_result(loaded("0x1", offline(), None)).view_state(now()).incoming_code, activate, "a failed load still lets the code reach the server");
        assert_eq!(single.on_refreshing().view_state(now()).incoming_code, None, "a refresh holds the code back");
        assert_eq!(
            single.on_refreshing().on_result(loaded("0x1", GemLoadState::Data, None)).view_state(now()).incoming_code,
            activate,
            "an unhandled code comes back after the refresh"
        );
        assert_eq!(single.on_code_handled().view_state(now()).incoming_code, None);

        let several = opened(&["0x1", "0x2"], Some("friend"));
        assert_eq!(several.view_state(now()).incoming_code, Some(GemIncomingCode::Confirm { code: "friend".to_string() }));
        assert_eq!(opened(&["0x1"], Some("  ")).view_state(now()).incoming_code, None);
    }

    #[test]
    fn test_a_code_answer_for_a_wallet_that_is_no_longer_shown_is_dropped() {
        let shown = opened(&["0x1", "0x2"], None).on_select_wallet(id("0x2"));
        let used = Rewards {
            used_referral_code: Some("friend".to_string()),
            ..invited()
        };

        assert_eq!(shown.on_rewards(id("0x1"), used.clone()), shown);
        assert_eq!(shown.on_rewards(id("0x2"), used.clone()).rewards, Some(used));
    }
}
