use crate::models::button::GemButtonState;
use crate::services::assets::model::GemFeeText;
use primitives::SimulationResult;

use super::error::{GemConfirmError, GemConfirmErrorSheet};
use super::model::{GemConfirmAction, GemConfirmButton, GemConfirmButtonKind, GemConfirmFailure, GemConfirmFeeValue, GemConfirmLoad, GemConfirmPhase, GemConfirmScreen, GemConfirmStage, GemTransferAmountResult};

impl GemConfirmScreen {
    pub fn initial(simulation: Option<&SimulationResult>) -> Self {
        Self {
            phase: GemConfirmPhase::Loading,
            has_critical_warning: simulation.is_some_and(SimulationResult::has_critical_warning),
            failure: None,
            has_fee: true,
            shown_sheet: None,
        }
    }

    fn load_sheet(&self) -> Option<GemConfirmErrorSheet> {
        self.failure.as_ref().filter(|failure| failure.stage == GemConfirmStage::Load).and_then(|failure| failure.error.display().sheet())
    }

    fn is_account_missing(&self) -> bool {
        self.failure.as_ref().is_some_and(|failure| failure.error.is_account_missing())
    }

    fn fee_text(&self, load: &GemConfirmLoad) -> Option<GemFeeText> {
        let fee = &load.fee.as_ref()?.formatted;
        let includes_network_fee = self.failure.as_ref().is_some_and(|failure| failure.error.includes_network_fee(&load.fee_asset.id));
        Some(match (includes_network_fee, load.shows_fee_assets()) {
            (true, _) => fee.text_with_amount(),
            (false, true) => fee.text_with_symbol(&load.fee_asset.symbol),
            (false, false) => fee.text(),
        })
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
    pub fn refreshes(&self) -> bool {
        self.phase == GemConfirmPhase::Ready
    }

    pub fn presents_sheet(&self) -> bool {
        self.load_sheet().is_some_and(|sheet| self.shown_sheet.as_ref() != Some(&sheet))
    }

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

    pub fn fee_value(&self, load: Option<GemConfirmLoad>) -> GemConfirmFeeValue {
        let unavailable = || GemConfirmFeeValue::Unavailable {
            text: crate::models::placeholder::EMPTY_VALUE.to_string(),
        };
        match self.phase {
            GemConfirmPhase::Loading => GemConfirmFeeValue::Loading,
            GemConfirmPhase::Failed => unavailable(),
            GemConfirmPhase::Ready | GemConfirmPhase::Confirming => match load.as_ref().and_then(|load| self.fee_text(load)) {
                Some(text) => GemConfirmFeeValue::Ready { text },
                None => unavailable(),
            },
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
        let shown_sheet = match self.phase {
            GemConfirmPhase::Loading => self.shown_sheet.clone(),
            GemConfirmPhase::Ready => self.load_sheet(),
            GemConfirmPhase::Confirming | GemConfirmPhase::Failed => None,
        };
        Self {
            phase: GemConfirmPhase::Loading,
            has_critical_warning: self.has_critical_warning,
            failure: None,
            has_fee: self.has_fee,
            shown_sheet,
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
            shown_sheet: self.shown_sheet.clone(),
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

    use super::super::model::{GemConfirmData, GemConfirmFee, GemFeeAsset};
    use super::super::rules::fee_asset_row;
    use super::*;
    use crate::formatted_number::GemFormattedNumber;
    use crate::models::placeholder::EMPTY_VALUE;
    use crate::services::assets::rules::fee_amount;
    use crate::services::balance::{GemAssetBalance, GemBalanceRequirement};
    use crate::services::localization::GemLocalizedText;
    use primitives::currency::Currency;

    #[test]
    fn test_a_load_error_with_an_info_sheet_presents_it() {
        let screen = GemConfirmScreen::initial(None);

        assert!(!screen.presents_sheet());
        assert!(screen.on_load_failed(GemConfirmError::ScanMalicious).presents_sheet());
        assert!(!screen.on_load_failed(GemConfirmError::Load { msg: "offline".into() }).presents_sheet());
        assert!(!screen.on_execute_failed(GemConfirmError::ScanMalicious).presents_sheet(), "an execute error stays in its row");
    }

    #[test]
    fn test_a_refresh_that_finds_the_same_problem_leaves_its_sheet_closed() {
        let network_fee_missing = |required: u64| GemConfirmError::InsufficientNetworkFee {
            asset: Asset::mock_eth(),
            requirement: Some(GemBalanceRequirement::new(required.into(), 0u64.into())),
        };
        let problem = |error: GemConfirmError| {
            let mut load = GemConfirmLoad::mock();
            load.fee = Some(GemConfirmFee::mock(GemTransferAmountResult::Error { error }));
            load
        };
        let shown = GemConfirmScreen::initial(None).on_load_started().on_loaded(problem(network_fee_missing(21_000)));
        assert!(shown.presents_sheet());

        let refreshed = shown.on_load_started().on_loaded(problem(network_fee_missing(42_000)));
        assert!(!refreshed.presents_sheet(), "a new fee changes the amounts, not the problem");
        assert!(
            !shown.on_load_started().on_load_started().on_loaded(problem(network_fee_missing(42_000))).presents_sheet(),
            "a fee change during a refresh is still a refresh"
        );

        let balance_missing = GemConfirmError::InsufficientBalance {
            asset: Asset::mock_eth(),
            requirement: GemBalanceRequirement::new(10u64.into(), 0u64.into()),
        };
        assert!(refreshed.on_load_started().on_loaded(problem(balance_missing)).presents_sheet(), "a different problem opens its own sheet");

        let resolved = refreshed.on_load_started().on_loaded(GemConfirmLoad::mock());
        assert!(
            resolved.on_load_started().on_loaded(problem(network_fee_missing(21_000))).presents_sheet(),
            "a problem that comes back after a load without it opens again"
        );
    }

    #[test]
    fn test_retry_opens_the_sheet_of_a_problem_that_is_still_there() {
        let failed = GemConfirmScreen::initial(None).on_load_failed(GemConfirmError::ScanMalicious);
        assert!(failed.presents_sheet());
        assert!(failed.on_load_started().on_load_failed(GemConfirmError::ScanMalicious).presents_sheet());
    }

    #[test]
    fn test_only_a_ready_screen_refreshes_on_the_timer() {
        let screen = GemConfirmScreen::initial(None);

        assert!(!screen.refreshes());
        assert!(
            GemConfirmScreen {
                phase: GemConfirmPhase::Ready,
                ..screen.clone()
            }
            .refreshes()
        );
        assert!(
            !GemConfirmScreen {
                phase: GemConfirmPhase::Confirming,
                ..screen
            }
            .refreshes()
        );
    }

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

        assert_eq!(waiting.fee_value(Some(GemConfirmLoad::mock())), GemConfirmFeeValue::Unavailable { text: EMPTY_VALUE.to_string() });
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
        let load = GemConfirmLoad {
            fee: Some(GemConfirmFee::mock(GemTransferAmountResult::mock())),
            ..GemConfirmLoad::mock()
        };
        let ready = GemConfirmFeeValue::Ready {
            text: GemConfirmFee::mock(GemTransferAmountResult::mock()).formatted.text(),
        };

        assert_eq!(loading.fee_value(Some(load.clone())), GemConfirmFeeValue::Loading);
        assert_eq!(
            GemConfirmScreen {
                phase: GemConfirmPhase::Ready,
                ..loading.clone()
            }
            .fee_value(Some(load.clone())),
            ready
        );
        assert_eq!(
            GemConfirmScreen {
                phase: GemConfirmPhase::Confirming,
                ..loading.clone()
            }
            .fee_value(Some(load.clone())),
            ready
        );
        assert_eq!(
            GemConfirmScreen { phase: GemConfirmPhase::Failed, ..loading }.fee_value(Some(load)),
            GemConfirmFeeValue::Unavailable {
                text: crate::models::placeholder::EMPTY_VALUE.to_string()
            },
            "a fee the screen could not load reads as the placeholder both apps use"
        );
    }

    #[test]
    fn test_confirm_fee_row_shows_the_fee_amount_only_when_a_balance_problem_includes_the_fee() {
        let eth = Asset::mock_eth();
        let priced = fee_amount(&eth, &num_bigint::BigInt::from(1_000_000_000_000_000u64), Some(2000.0), Currency::USD);
        let fiat = priced.fiat.clone().unwrap();
        let fee = |amount: GemTransferAmountResult| GemConfirmFee {
            formatted: priced.clone(),
            ..GemConfirmFee::mock(amount)
        };
        let short_of = |asset: Asset| GemTransferAmountResult::Error {
            error: GemConfirmError::InsufficientBalance {
                asset,
                requirement: GemBalanceRequirement::new(10u64.into(), 0u64.into()),
            },
        };
        let row = |load: GemConfirmLoad| GemConfirmScreen::initial(None).on_loaded(load.clone()).fee_value(Some(load));
        let load = |amount: GemTransferAmountResult| GemConfirmLoad {
            fee: Some(fee(amount)),
            ..GemConfirmLoad::mock()
        };
        let text = |value: GemFormattedNumber, extra: Option<GemLocalizedText>| GemConfirmFeeValue::Ready { text: GemFeeText { value, extra } };
        let btc = Asset::mock_btc();
        let picker = GemConfirmLoad {
            fee_assets: vec![GemFeeAsset {
                asset: btc.clone(),
                balance: GemAssetBalance::mock_with_available(1),
                price: None,
                row: fee_asset_row(&btc, &GemAssetBalance::mock_with_available(1), None, &Currency::USD),
            }],
            ..load(GemTransferAmountResult::mock())
        };

        assert_eq!(row(load(GemTransferAmountResult::mock())), text(fiat.clone(), None), "a fee reads in the user's currency only");
        assert_eq!(
            row(load(short_of(eth.clone()))),
            text(priced.amount.clone(), Some(GemLocalizedText::Number { number: fiat.clone() })),
            "a shortfall in the asset that pays the fee shows the fee amount the user has to cover"
        );
        assert_eq!(row(load(short_of(Asset::mock_ethereum_usdc()))), text(fiat.clone(), None), "a token shortfall does not involve the fee");
        assert_eq!(
            row(picker.clone()),
            text(fiat.clone(), Some(GemLocalizedText::Text { text: eth.symbol.clone() })),
            "a fee paid in a picked asset names that asset"
        );
        assert_eq!(
            row(GemConfirmLoad {
                fee: Some(GemConfirmFee::mock(GemTransferAmountResult::mock())),
                ..picker
            }),
            text(GemConfirmFee::mock(GemTransferAmountResult::mock()).formatted.amount, None),
            "a fee without a price shows its amount, which already names the asset"
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
