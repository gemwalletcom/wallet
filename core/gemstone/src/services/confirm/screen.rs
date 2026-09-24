use crate::models::button::GemButtonState;
use primitives::SimulationResult;

use super::error::GemConfirmError;
use super::model::{GemConfirmAction, GemConfirmButton, GemConfirmButtonKind, GemConfirmFailure, GemConfirmFeeRow, GemConfirmLoad, GemConfirmPhase, GemConfirmScreen, GemConfirmStage, GemTransferAmountResult};

impl GemConfirmScreen {
    pub fn initial(simulation: Option<&SimulationResult>) -> Self {
        Self {
            phase: GemConfirmPhase::Loading,
            has_critical_warning: simulation.is_some_and(SimulationResult::has_critical_warning),
            failure: None,
            has_fee: true,
        }
    }

    fn is_account_missing(&self) -> bool {
        self.failure.as_ref().is_some_and(|failure| failure.error.is_account_missing())
    }

    fn failed(&self, stage: GemConfirmStage, error: GemConfirmError) -> Self {
        Self {
            phase: GemConfirmPhase::Failed,
            failure: Some(GemConfirmFailure { stage, error }),
            ..self.clone()
        }
    }
}

#[uniffi::export]
impl GemConfirmScreen {
    pub fn button(&self) -> GemConfirmButton {
        let button = |kind, state| GemConfirmButton { kind, state };
        match self.phase {
            GemConfirmPhase::Loading | GemConfirmPhase::Confirming => button(GemConfirmButtonKind::Confirm, GemButtonState::Loading),
            GemConfirmPhase::Failed if self.is_account_missing() => button(GemConfirmButtonKind::AccountMissing, GemButtonState::Disabled),
            GemConfirmPhase::Failed => button(GemConfirmButtonKind::Retry, GemButtonState::Enabled),
            GemConfirmPhase::Ready if !self.has_fee || self.failure.is_some() || self.has_critical_warning => button(GemConfirmButtonKind::Confirm, GemButtonState::Disabled),
            GemConfirmPhase::Ready => button(GemConfirmButtonKind::Confirm, GemButtonState::Enabled),
        }
    }

    pub fn fee_row(&self) -> GemConfirmFeeRow {
        match self.phase {
            GemConfirmPhase::Loading => GemConfirmFeeRow::Loading,
            GemConfirmPhase::Failed => GemConfirmFeeRow::Unavailable {
                text: crate::models::placeholder::EMPTY_VALUE.to_string(),
            },
            GemConfirmPhase::Ready | GemConfirmPhase::Confirming if !self.has_fee => GemConfirmFeeRow::Unavailable {
                text: crate::models::placeholder::EMPTY_VALUE.to_string(),
            },
            GemConfirmPhase::Ready | GemConfirmPhase::Confirming => GemConfirmFeeRow::Ready,
        }
    }

    pub fn action(&self) -> Option<GemConfirmAction> {
        match self.phase {
            GemConfirmPhase::Loading | GemConfirmPhase::Confirming => None,
            GemConfirmPhase::Failed if self.is_account_missing() => None,
            GemConfirmPhase::Failed => Some(GemConfirmAction::Load),
            GemConfirmPhase::Ready if !self.has_fee || self.failure.is_some() => None,
            GemConfirmPhase::Ready => Some(GemConfirmAction::Execute),
        }
    }

    pub fn on_load_started(&self) -> GemConfirmScreen {
        Self {
            phase: GemConfirmPhase::Loading,
            has_critical_warning: self.has_critical_warning,
            failure: None,
            has_fee: self.has_fee,
        }
    }

    pub fn on_loaded(&self, load: GemConfirmLoad) -> GemConfirmScreen {
        let amount_error = load.fee.as_ref().and_then(|fee| match &fee.amount {
            GemTransferAmountResult::Error { error } => Some(error.clone()),
            GemTransferAmountResult::Amount { .. } => None,
        });
        Self {
            phase: GemConfirmPhase::Ready,
            has_critical_warning: load.simulation.simulation.as_ref().is_some_and(|simulation| simulation.has_critical_warning),
            failure: amount_error.map(|error| GemConfirmFailure { stage: GemConfirmStage::Load, error }),
            has_fee: load.fee.is_some(),
        }
    }

    pub fn on_load_failed(&self, error: GemConfirmError) -> GemConfirmScreen {
        self.failed(GemConfirmStage::Load, error)
    }

    pub fn on_execute_started(&self) -> GemConfirmScreen {
        Self {
            phase: GemConfirmPhase::Confirming,
            failure: None,
            ..self.clone()
        }
    }

    pub fn on_execute_cancelled(&self) -> GemConfirmScreen {
        Self {
            phase: GemConfirmPhase::Ready,
            failure: None,
            ..self.clone()
        }
    }

    pub fn on_execute_failed(&self, error: GemConfirmError) -> GemConfirmScreen {
        self.failed(GemConfirmStage::Execute, error)
    }
}

#[cfg(test)]
mod tests {
    use primitives::{Asset, Chain, SimulationResult, SimulationWarning, TransactionInputType, TransferAmount};

    use super::super::model::{GemConfirmData, GemConfirmFee};
    use super::*;
    use crate::models::placeholder::EMPTY_VALUE;

    #[test]
    fn test_a_missing_account_offers_no_retry() {
        let missing = GemConfirmScreen::initial(None).on_load_failed(GemConfirmError::AccountMissing { chain: Chain::Tron });

        assert_eq!(
            missing.button(),
            GemConfirmButton {
                kind: GemConfirmButtonKind::AccountMissing,
                state: GemButtonState::Disabled
            }
        );
        assert_eq!(missing.action(), None);
        assert_eq!(GemConfirmScreen::initial(None).on_load_failed(GemConfirmError::Offline).action(), Some(GemConfirmAction::Load));
    }

    #[test]
    fn test_confirm_button_follows_the_phase_and_the_ready_checks() {
        let ready = GemConfirmScreen {
            phase: GemConfirmPhase::Ready,
            ..GemConfirmScreen::initial(None)
        };

        assert_eq!(
            GemConfirmScreen::initial(None).button(),
            GemConfirmButton {
                kind: GemConfirmButtonKind::Confirm,
                state: GemButtonState::Loading
            }
        );
        assert_eq!(
            GemConfirmScreen {
                phase: GemConfirmPhase::Confirming,
                ..ready.clone()
            }
            .button(),
            GemConfirmButton {
                kind: GemConfirmButtonKind::Confirm,
                state: GemButtonState::Loading
            }
        );
        assert_eq!(
            GemConfirmScreen {
                phase: GemConfirmPhase::Failed,
                has_critical_warning: true,
                ..ready.clone()
            }
            .button(),
            GemConfirmButton {
                kind: GemConfirmButtonKind::Retry,
                state: GemButtonState::Enabled
            }
        );
        assert_eq!(
            GemConfirmScreen {
                failure: Some(GemConfirmFailure {
                    stage: GemConfirmStage::Load,
                    error: GemConfirmError::Load { msg: "down".to_string() },
                }),
                ..ready.clone()
            }
            .button(),
            GemConfirmButton {
                kind: GemConfirmButtonKind::Confirm,
                state: GemButtonState::Disabled
            }
        );
        assert_eq!(
            GemConfirmScreen { has_critical_warning: true, ..ready.clone() }.button(),
            GemConfirmButton {
                kind: GemConfirmButtonKind::Confirm,
                state: GemButtonState::Disabled
            }
        );
        assert_eq!(
            ready.button(),
            GemConfirmButton {
                kind: GemConfirmButtonKind::Confirm,
                state: GemButtonState::Enabled
            }
        );
    }

    #[test]
    fn test_a_screen_without_a_preload_cannot_be_confirmed() {
        let waiting = GemConfirmScreen {
            phase: GemConfirmPhase::Ready,
            has_fee: false,
            ..GemConfirmScreen::initial(None)
        };

        assert_eq!(waiting.fee_row(), GemConfirmFeeRow::Unavailable { text: EMPTY_VALUE.to_string() });
        assert_eq!(
            waiting.button(),
            GemConfirmButton {
                kind: GemConfirmButtonKind::Confirm,
                state: GemButtonState::Disabled
            }
        );
        assert_eq!(waiting.action(), None);
        assert!(!GemConfirmScreen::initial(None).on_loaded(GemConfirmLoad::mock()).has_fee);
    }

    #[test]
    fn test_confirm_fee_row_waits_for_the_preload_and_gives_up_on_failure() {
        let loading = GemConfirmScreen {
            has_critical_warning: true,
            ..GemConfirmScreen::initial(None)
        };

        assert_eq!(loading.fee_row(), GemConfirmFeeRow::Loading);
        assert_eq!(
            GemConfirmScreen {
                phase: GemConfirmPhase::Ready,
                ..loading.clone()
            }
            .fee_row(),
            GemConfirmFeeRow::Ready
        );
        assert_eq!(
            GemConfirmScreen {
                phase: GemConfirmPhase::Confirming,
                ..loading.clone()
            }
            .fee_row(),
            GemConfirmFeeRow::Ready
        );
        assert_eq!(
            GemConfirmScreen { phase: GemConfirmPhase::Failed, ..loading }.fee_row(),
            GemConfirmFeeRow::Unavailable {
                text: crate::models::placeholder::EMPTY_VALUE.to_string()
            },
            "a fee the screen could not load reads as the placeholder both apps use"
        );
    }

    #[test]
    fn test_confirm_screen_transitions_follow_the_load_and_execute_outcomes() {
        let started = GemConfirmScreen::initial(None).on_load_started();
        assert_eq!(started.phase, GemConfirmPhase::Loading);
        assert_eq!(started.action(), None);

        let mut load = GemConfirmLoad::mock();
        load.fee = Some(GemConfirmFee::mock(GemTransferAmountResult::mock()));
        let ready = started.on_loaded(load.clone());
        assert_eq!(ready.phase, GemConfirmPhase::Ready);
        assert!(ready.failure.is_none());
        assert_eq!(ready.action(), Some(GemConfirmAction::Execute));

        load.fee = Some(GemConfirmFee::mock(GemTransferAmountResult::Error {
            error: GemConfirmError::Load { msg: "down".to_string() },
        }));
        let amount_failed = started.on_loaded(load);
        assert_eq!(amount_failed.phase, GemConfirmPhase::Ready);
        assert_eq!(amount_failed.failure.as_ref().map(|failure| failure.stage), Some(GemConfirmStage::Load));
        assert_eq!(
            amount_failed.button(),
            GemConfirmButton {
                kind: GemConfirmButtonKind::Confirm,
                state: GemButtonState::Disabled,
            }
        );
        assert_eq!(amount_failed.action(), None, "an amount the wallet cannot cover is not retried by pressing the button");

        let load_failed = started.on_load_failed(GemConfirmError::Load { msg: "down".to_string() });
        assert_eq!(load_failed.phase, GemConfirmPhase::Failed);
        assert_eq!(load_failed.failure.as_ref().map(|failure| failure.stage), Some(GemConfirmStage::Load));
        assert_eq!(load_failed.action(), Some(GemConfirmAction::Load));
        assert!(load_failed.on_load_started().failure.is_none());

        let confirming = ready.on_execute_started();
        assert_eq!(confirming.phase, GemConfirmPhase::Confirming);
        assert_eq!(confirming.action(), None);
        assert_eq!(confirming.on_execute_cancelled().phase, GemConfirmPhase::Ready);

        let execute_failed = confirming.on_execute_failed(GemConfirmError::Broadcast { hashes: vec![], msg: "rejected".to_string() });
        assert_eq!(execute_failed.phase, GemConfirmPhase::Failed);
        assert_eq!(execute_failed.failure.as_ref().map(|failure| failure.stage), Some(GemConfirmStage::Execute));
        assert_eq!(execute_failed.action(), Some(GemConfirmAction::Load));
        assert_eq!(execute_failed.button().kind, GemConfirmButtonKind::Retry);
    }

    #[test]
    fn test_initial_screen_reads_the_critical_warning_off_the_request_simulation() {
        let critical = SimulationResult {
            warnings: vec![SimulationWarning::validation_error("bad")],
            ..SimulationResult::default()
        };
        assert_eq!(GemConfirmScreen::initial(Some(&critical)).has_critical_warning, critical.has_critical_warning());
        assert!(!GemConfirmScreen::initial(None).has_critical_warning);
        assert_eq!(GemConfirmScreen::initial(None).phase, GemConfirmPhase::Loading);
    }

    #[test]
    fn test_retry_reloads_max_transfer_before_confirming() {
        let max_transfer = |fee: u64, value: u64| {
            let mut load = GemConfirmLoad::mock();
            load.metadata.asset_balance.available = 1_000_000u64.into();
            load.metadata.fee_asset_balance.available = 1_000_000u64.into();
            let mut data = GemConfirmData::mock(Chain::Ethereum, TransactionInputType::Transfer { asset: Asset::mock_eth() });
            data.input.transfer.value = 1_000_000u64.into();
            data.fee.fee = fee.into();
            data.fee.fee_asset = load.fee_asset.id.clone();
            let amount = data.preload_amount(&load.metadata, &load.fee_asset).unwrap();
            match &amount {
                GemTransferAmountResult::Amount { amount } => assert_eq!(
                    amount,
                    &TransferAmount {
                        value: value.into(),
                        network_fee: fee.into(),
                        is_max_amount: true,
                    }
                ),
                GemTransferAmountResult::Error { error } => panic!("unexpected max transfer error: {error}"),
            }
            load.fee = Some(GemConfirmFee::mock(amount));
            load
        };
        let ready = GemConfirmScreen::initial(None).on_loaded(max_transfer(21_000, 979_000));
        let error = GemConfirmError::Network { msg: "rejected".to_string() };
        let failed = ready.on_execute_started().on_execute_failed(error);

        assert_eq!(failed.action(), Some(GemConfirmAction::Load));
        assert_eq!(failed.button().kind, GemConfirmButtonKind::Retry);

        let loading = failed.on_load_started();
        assert_eq!(loading.action(), None);
        assert_eq!(loading.on_load_failed(GemConfirmError::Offline).action(), Some(GemConfirmAction::Load));

        let ready = loading.on_loaded(max_transfer(42_000, 958_000));
        assert_eq!(ready.button().kind, GemConfirmButtonKind::Confirm);
        assert_eq!(ready.action(), Some(GemConfirmAction::Execute));
    }

    #[test]
    fn test_every_execution_failure_reloads_before_retry() {
        let errors = [
            GemConfirmError::Offline,
            GemConfirmError::Network { msg: "request timed out".to_string() },
            GemConfirmError::Broadcast { hashes: vec![], msg: "rejected".to_string() },
            GemConfirmError::Broadcast {
                hashes: vec!["accepted-transaction".to_string()],
                msg: "rejected".to_string(),
            },
            GemConfirmError::Record { msg: "store unavailable".to_string() },
        ];
        for error in errors {
            assert_eq!(GemConfirmScreen::initial(None).on_execute_failed(error).action(), Some(GemConfirmAction::Load));
        }
    }
}
