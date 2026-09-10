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
            amount_failed: false,
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
            GemConfirmPhase::Ready if self.amount_failed => button(GemConfirmButtonKind::Retry, GemConfirmButtonState::Enabled),
            GemConfirmPhase::Ready if self.has_critical_warning => button(GemConfirmButtonKind::Confirm, GemConfirmButtonState::Disabled),
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
            GemConfirmPhase::Failed => match &self.failure {
                Some(failure) if failure.stage == GemConfirmStage::Execute => Some(GemConfirmAction::Execute),
                _ => Some(GemConfirmAction::Load),
            },
            GemConfirmPhase::Ready if self.amount_failed => Some(GemConfirmAction::Load),
            GemConfirmPhase::Ready => Some(GemConfirmAction::Execute),
        }
    }

    pub fn on_load_started(&self) -> GemConfirmScreen {
        Self {
            phase: GemConfirmPhase::Loading,
            amount_failed: false,
            has_critical_warning: self.has_critical_warning,
            failure: None,
        }
    }

    pub fn on_loaded(&self, load: GemConfirmLoad) -> GemConfirmScreen {
        Self {
            phase: GemConfirmPhase::Ready,
            amount_failed: load.preload.as_ref().is_some_and(|preload| matches!(preload.amount, GemTransferAmountResult::Error { .. })),
            has_critical_warning: load.simulation.simulation.as_ref().is_some_and(|simulation| simulation.has_critical_warning),
            failure: None,
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
    use primitives::{Account, Asset, Chain, SimulationResult, SimulationWarning, TransactionInputType};

    use super::super::model::{GemConfirmMetadata, GemConfirmPreload, GemConfirmSimulationState};
    use crate::services::transfer::{GemRecipient, GemTransferData};
    use super::super::testkit::confirm_data;
    use super::*;
    use crate::services::balance::GemAssetBalance;

    #[test]
    fn test_confirm_button_follows_the_phase_and_the_ready_checks() {
        let screen = |phase, amount_failed, has_critical_warning| GemConfirmScreen {
            phase,
            amount_failed,
            has_critical_warning,
            failure: None,
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
            screen(GemConfirmPhase::Ready, true, true).button(),
            button(GemConfirmButtonKind::Retry, GemConfirmButtonState::Enabled)
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
            amount_failed: true,
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
        assert!(!ready.amount_failed);
        assert_eq!(ready.action(), Some(GemConfirmAction::Execute));

        load.preload = Some(GemConfirmPreload {
            confirm_data: confirm_data(Chain::Ethereum, TransactionInputType::Transfer { asset: Asset::mock_eth() }, "sender"),
            amount: GemTransferAmountResult::Error { error: failed_load() },
        });
        let amount_failed = started.on_loaded(load);
        assert!(amount_failed.amount_failed);
        assert_eq!(amount_failed.action(), Some(GemConfirmAction::Load));

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
        assert_eq!(execute_failed.action(), Some(GemConfirmAction::Execute));
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

    fn load_without_preload() -> GemConfirmLoad {
        let eth = Asset::mock_eth();
        GemConfirmLoad {
            transfer: GemTransferData {
                input_type: TransactionInputType::Transfer { asset: eth.clone() },
                recipient: GemRecipient::address("recipient".into()),
                value: 1.into(),
                use_max_amount: false,
            },
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
