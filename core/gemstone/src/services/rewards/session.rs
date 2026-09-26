use chrono::{DateTime, Utc};
use primitives::{Rewards, WalletId};

use super::model::{GemRewardsResult, GemRewardsViewState};
use super::rules;
use crate::models::state::{GemLoad, GemLoadState};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRewardsSession {
    pub wallet_id: Option<WalletId>,
    pub state: GemLoadState,
    pub rewards: Option<Rewards>,
    pub is_refreshing: bool,
}

#[uniffi::export]
impl GemRewardsSession {
    pub fn on_select_wallet(&self, wallet_id: WalletId) -> Self {
        if self.wallet_id.as_ref() == Some(&wallet_id) {
            return self.clone();
        }
        Self {
            wallet_id: Some(wallet_id),
            state: GemLoadState::Loading,
            rewards: None,
            is_refreshing: false,
        }
    }

    pub fn on_refreshing(&self) -> Self {
        Self { is_refreshing: true, ..self.clone() }
    }

    pub fn on_result(&self, result: GemRewardsResult) -> Self {
        if self.wallet_id.as_ref() != Some(&result.wallet_id) {
            return self.clone();
        }
        let shown = self.shown().data(result.state.into_result(result.rewards));
        Self {
            wallet_id: self.wallet_id.clone(),
            state: shown.state,
            rewards: shown.value,
            is_refreshing: false,
        }
    }

    pub fn on_rewards(&self, rewards: Rewards) -> Self {
        Self {
            state: GemLoadState::Data,
            rewards: Some(rewards),
            is_refreshing: false,
            ..self.clone()
        }
    }

    pub fn view_state(&self, now: DateTime<Utc>) -> GemRewardsViewState {
        GemRewardsViewState {
            state: self.state.clone(),
            rewards: rules::state(self.rewards.as_ref(), now),
            is_refreshing: self.is_refreshing,
        }
    }
}

impl GemRewardsSession {
    fn shown(&self) -> GemLoad<Option<Rewards>> {
        GemLoad {
            state: self.state.clone(),
            value: self.rewards.clone(),
        }
    }
}

#[uniffi::export]
pub fn rewards_session() -> GemRewardsSession {
    GemRewardsSession {
        wallet_id: None,
        state: GemLoadState::Loading,
        rewards: None,
        is_refreshing: false,
    }
}

#[cfg(test)]
mod tests {
    use primitives::RewardStatus;

    use super::*;
    use crate::services::error::GemServiceError;

    fn failed() -> GemRewardsResult {
        GemRewardsResult {
            wallet_id: wallet(),
            state: GemLoadState::Error { error: offline() },
            rewards: None,
        }
    }

    fn now() -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000, 0).unwrap()
    }

    fn wallet() -> WalletId {
        WalletId::Multicoin("0x1".to_string())
    }

    fn offline() -> GemServiceError {
        GemServiceError::Gateway { msg: "offline".to_string() }
    }

    fn invited() -> Rewards {
        Rewards {
            code: Some("GEM123".to_string()),
            ..Rewards::mock(None, RewardStatus::Verified)
        }
    }

    #[test]
    fn test_a_failed_load_reads_as_a_failure_instead_of_a_wallet_without_a_code() {
        let session = rewards_session().on_select_wallet(wallet());

        let unreachable = session.on_result(failed());

        assert!(matches!(unreachable.view_state(now()).state, GemLoadState::Error { .. }), "a wallet with a code must not be offered the create-code screen");
        assert!(unreachable.view_state(now()).rewards.actions.is_empty(), "a failed wallet is offered nothing to do");
    }

    #[test]
    fn test_a_failed_refresh_keeps_the_code_already_on_screen() {
        let shown = rewards_session().on_select_wallet(wallet()).on_rewards(invited());

        let kept = shown.on_result(failed());

        assert_eq!(kept.view_state(now()).state, GemLoadState::Data);
        assert_eq!(kept.view_state(now()).rewards, shown.view_state(now()).rewards);
    }

    #[test]
    fn test_a_result_for_a_wallet_that_is_no_longer_shown_is_dropped() {
        let second = WalletId::Multicoin("0x2".to_string());
        let shown = rewards_session().on_select_wallet(second);
        let late = GemRewardsResult {
            wallet_id: wallet(),
            state: GemLoadState::Data,
            rewards: Some(invited()),
        };

        assert_eq!(shown.on_result(late), shown, "the wallet moved on before the answer arrived");
    }

    #[test]
    fn test_selecting_another_wallet_starts_over_and_reselecting_the_same_one_does_not() {
        let shown = rewards_session().on_select_wallet(wallet()).on_rewards(invited());

        assert_eq!(shown.on_select_wallet(wallet()), shown);
        let switched = shown.on_select_wallet(WalletId::Multicoin("0x2".to_string()));
        assert_eq!(switched.view_state(now()).state, GemLoadState::Loading);
        assert_eq!(switched.wallet_id, Some(WalletId::Multicoin("0x2".to_string())));
    }

    #[test]
    fn test_a_refresh_shows_until_its_result_arrives() {
        let wallet_id = WalletId::Multicoin("wallet".into());
        let session = rewards_session().on_select_wallet(wallet_id.clone()).on_refreshing();
        assert!(session.view_state(Utc::now()).is_refreshing);

        let loaded = session.on_result(GemRewardsResult {
            wallet_id,
            state: GemLoadState::Data,
            rewards: None,
        });
        assert!(!loaded.view_state(Utc::now()).is_refreshing);
    }
}
