use primitives::SimulationResult;

use super::error::GemConfirmError;
use super::model::{
    GemConfirmAction, GemConfirmButton, GemConfirmButtonKind, GemConfirmButtonState, GemConfirmFailure, GemConfirmFeeRow, GemConfirmLoad, GemConfirmPhase, GemConfirmScreen,
    GemConfirmStage, GemTransferAmountResult,
};

impl GemConfirmScreen {
    pub fn initial(simulation: Option<&SimulationResult>) -> Self {
        Self {
            phase: GemConfirmPhase::Loading,
            has_critical_warning: simulation.is_some_and(SimulationResult::has_critical_warning),
            failure: None,
        }
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
            GemConfirmPhase::Loading | GemConfirmPhase::Confirming => button(GemConfirmButtonKind::Confirm, GemConfirmButtonState::Loading),
            GemConfirmPhase::Failed => button(GemConfirmButtonKind::Retry, GemConfirmButtonState::Enabled),
            GemConfirmPhase::Ready if self.failure.is_some() || self.has_critical_warning => button(GemConfirmButtonKind::Confirm, GemConfirmButtonState::Disabled),
            GemConfirmPhase::Ready => button(GemConfirmButtonKind::Confirm, GemConfirmButtonState::Enabled),
        }
    }

    pub fn fee_row(&self) -> GemConfirmFeeRow {
        match self.phase {
            GemConfirmPhase::Loading => GemConfirmFeeRow::Loading,
            GemConfirmPhase::Failed => GemConfirmFeeRow::Unavailable,
            GemConfirmPhase::Ready | GemConfirmPhase::Confirming => GemConfirmFeeRow::Ready,
        }
    }

    pub fn action(&self) -> Option<GemConfirmAction> {
        match self.phase {
            GemConfirmPhase::Loading | GemConfirmPhase::Confirming => None,
            GemConfirmPhase::Failed => Some(GemConfirmAction::Load),
            GemConfirmPhase::Ready if self.failure.is_some() => None,
            GemConfirmPhase::Ready => Some(GemConfirmAction::Execute),
        }
    }

    pub fn on_load_started(&self) -> GemConfirmScreen {
        Self {
            phase: GemConfirmPhase::Loading,
            has_critical_warning: self.has_critical_warning,
            failure: None,
        }
    }

    pub fn on_loaded(&self, load: GemConfirmLoad) -> GemConfirmScreen {
        let amount_error = load.preload.as_ref().and_then(|preload| match &preload.amount {
            GemTransferAmountResult::Error { error } => Some(error.clone()),
            GemTransferAmountResult::Amount { .. } => None,
        });
        Self {
            phase: GemConfirmPhase::Ready,
            has_critical_warning: load.simulation.simulation.as_ref().is_some_and(|simulation| simulation.has_critical_warning),
            failure: amount_error.map(|error| GemConfirmFailure {
                stage: GemConfirmStage::Load,
                error,
            }),
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
    use primitives::{Account, Asset, Chain, SimulationResult, SimulationWarning, TransactionInputType, TransferAmount};

    use super::super::model::{GemConfirmMetadata, GemConfirmPreload, GemConfirmSimulationState};
    use super::super::testkit::confirm_data;
    use super::*;
    use crate::services::balance::GemAssetBalance;

    #[test]
    fn test_confirm_button_follows_the_phase_and_the_ready_checks() {
        let screen = |phase, failed: bool, has_critical_warning| GemConfirmScreen {
            phase,
            has_critical_warning,
            failure: failed.then(|| GemConfirmFailure {
                stage: GemConfirmStage::Load,
                error: GemConfirmError::Load { msg: "down".to_string() },
            }),
        };
        let button = |kind, state| GemConfirmButton { kind, state };

        assert_eq!(
            screen(GemConfirmPhase::Loading, false, false).button(),
            button(GemConfirmButtonKind::Confirm, GemConfirmButtonState::Loading)
        );
        assert_eq!(
            screen(GemConfirmPhase::Confirming, false, false).button(),
            button(GemConfirmButtonKind::Confirm, GemConfirmButtonState::Loading)
        );
        assert_eq!(
            screen(GemConfirmPhase::Failed, false, true).button(),
            button(GemConfirmButtonKind::Retry, GemConfirmButtonState::Enabled)
        );
        assert_eq!(
            screen(GemConfirmPhase::Ready, true, false).button(),
            button(GemConfirmButtonKind::Confirm, GemConfirmButtonState::Disabled)
        );
        assert_eq!(
            screen(GemConfirmPhase::Ready, false, true).button(),
            button(GemConfirmButtonKind::Confirm, GemConfirmButtonState::Disabled)
        );
        assert_eq!(
            screen(GemConfirmPhase::Ready, false, false).button(),
            button(GemConfirmButtonKind::Confirm, GemConfirmButtonState::Enabled)
        );
    }

    #[test]
    fn test_confirm_fee_row_waits_for_the_preload_and_gives_up_on_failure() {
        let screen = |phase| GemConfirmScreen {
            phase,
            has_critical_warning: true,
            failure: None,
        };

        assert_eq!(screen(GemConfirmPhase::Loading).fee_row(), GemConfirmFeeRow::Loading);
        assert_eq!(screen(GemConfirmPhase::Ready).fee_row(), GemConfirmFeeRow::Ready);
        assert_eq!(screen(GemConfirmPhase::Confirming).fee_row(), GemConfirmFeeRow::Ready);
        assert_eq!(screen(GemConfirmPhase::Failed).fee_row(), GemConfirmFeeRow::Unavailable);
    }

    #[test]
    fn test_confirm_screen_transitions_follow_the_load_and_execute_outcomes() {
        let failed_load = || GemConfirmError::Load { msg: "down".to_string() };
        let started = GemConfirmScreen::initial(None).on_load_started();
        assert_eq!(started.phase, GemConfirmPhase::Loading);
        assert_eq!(started.action(), None);

        let mut load = load_without_preload();
        let ready = started.on_loaded(load.clone());
        assert_eq!(ready.phase, GemConfirmPhase::Ready);
        assert!(ready.failure.is_none());
        assert_eq!(ready.action(), Some(GemConfirmAction::Execute));

        load.preload = Some(GemConfirmPreload {
            confirm_data: confirm_data(Chain::Ethereum, TransactionInputType::Transfer { asset: Asset::mock_eth() }, "sender"),
            amount: GemTransferAmountResult::Error { error: failed_load() },
        });
        let amount_failed = started.on_loaded(load);
        assert_eq!(amount_failed.phase, GemConfirmPhase::Ready);
        assert_eq!(amount_failed.failure.as_ref().map(|failure| failure.stage), Some(GemConfirmStage::Load));
        assert_eq!(
            amount_failed.button(),
            GemConfirmButton {
                kind: GemConfirmButtonKind::Confirm,
                state: GemConfirmButtonState::Disabled,
            }
        );
        assert_eq!(amount_failed.action(), None, "an amount the wallet cannot cover is not retried by pressing the button");

        let load_failed = started.on_load_failed(failed_load());
        assert_eq!(load_failed.phase, GemConfirmPhase::Failed);
        assert_eq!(load_failed.failure.as_ref().map(|failure| failure.stage), Some(GemConfirmStage::Load));
        assert_eq!(load_failed.action(), Some(GemConfirmAction::Load));
        assert!(load_failed.on_load_started().failure.is_none());

        let confirming = ready.on_execute_started();
        assert_eq!(confirming.phase, GemConfirmPhase::Confirming);
        assert_eq!(confirming.action(), None);
        assert_eq!(confirming.on_execute_cancelled().phase, GemConfirmPhase::Ready);

        let execute_failed = confirming.on_execute_failed(GemConfirmError::Broadcast {
            hashes: vec![],
            msg: "rejected".to_string(),
        });
        assert_eq!(execute_failed.phase, GemConfirmPhase::Failed);
        assert_eq!(execute_failed.failure.as_ref().map(|failure| failure.stage), Some(GemConfirmStage::Execute));
        assert_eq!(execute_failed.action(), Some(GemConfirmAction::Load));
        assert_eq!(execute_failed.button().kind, GemConfirmButtonKind::Retry);
    }

    #[test]
    fn test_initial_screen_reads_the_critical_warning_off_the_request_simulation() {
        let critical = SimulationResult {
            warnings: vec![SimulationWarning::validation_error("bad")],
            balance_changes: vec![],
            payload: vec![],
            header: None,
        };
        assert_eq!(GemConfirmScreen::initial(Some(&critical)).has_critical_warning, critical.has_critical_warning());
        assert!(!GemConfirmScreen::initial(None).has_critical_warning);
        assert_eq!(GemConfirmScreen::initial(None).phase, GemConfirmPhase::Loading);
    }

    #[test]
    fn test_retry_reloads_max_transfer_before_confirming() {
        let max_transfer = |fee: u64, value: u64| {
            let mut load = load_without_preload();
            load.metadata.asset_balance.available = 1_000_000u64.into();
            load.metadata.fee_asset_balance.available = 1_000_000u64.into();
            let mut data = confirm_data(Chain::Ethereum, TransactionInputType::Transfer { asset: Asset::mock_eth() }, "sender");
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
            load.preload = Some(GemConfirmPreload { confirm_data: data, amount });
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
            GemConfirmError::Network {
                msg: "request timed out".to_string(),
            },
            GemConfirmError::Broadcast {
                hashes: vec![],
                msg: "rejected".to_string(),
            },
            GemConfirmError::Broadcast {
                hashes: vec!["accepted-transaction".to_string()],
                msg: "rejected".to_string(),
            },
            GemConfirmError::Record {
                msg: "store unavailable".to_string(),
            },
        ];
        for error in errors {
            assert_eq!(GemConfirmScreen::initial(None).on_execute_failed(error).action(), Some(GemConfirmAction::Load));
        }
    }

    fn load_without_preload() -> GemConfirmLoad {
        let eth = Asset::mock_eth();
        GemConfirmLoad {
            sender: Account::mock(Chain::Ethereum, "sender"),
            fee_asset: eth.clone(),
            metadata: GemConfirmMetadata {
                asset_balance: GemAssetBalance::mock(),
                fee_asset_balance: GemAssetBalance::mock(),
                prices: vec![],
            },
            fee_assets: vec![],
            simulation: GemConfirmSimulationState {
                chain: Chain::Ethereum,
                result: None,
                warnings: vec![],
                simulation: None,
                address_names: vec![],
            },
            address_name: None,
            preload: None,
        }
    }
}
