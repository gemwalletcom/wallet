use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use primitives::{AddressName, AssetId, ChainAddress, PaymentVerification, PerpetualType, SimulationResult, TransactionInputType, Wallet};

use super::error::{GemConfirmErrorInfo, SendFailure};
use super::header::{self, GemConfirmHeader};
use super::rules::{asset_pick_needs_reload, preload_simulation};
use super::{
    ConfirmState, ConfirmSwapQuote, GemConfirmDetails, GemConfirmError, GemConfirmFeeLoad, GemConfirmFeeRow, GemConfirmInput, GemConfirmLoad, GemConfirmLoadOptions, GemConfirmRowContent, GemConfirmScreen, GemConfirmStage,
    GemConfirmTransferService, GemConfirmViewState, GemNetworkFeeScreen, GemSubmitResult, GemTransferAmountResult, SendInput, SignedSubmission,
};
use crate::models::list::GemListRow;
use crate::payment::GemPaymentLoad;
use crate::services::amount::model::GemNumberFormat;
use crate::services::perpetual::model::perpetual_confirm_details;
use crate::services::perpetual::rules::autoclose_row;
use crate::services::simulation::warning_rows;
use crate::services::swap::model::swap_quote_details;
use crate::services::transfer::GemTransferData;
use crate::services::wallet::GemKeystoreAuthentication;

#[derive(uniffi::Object)]
pub struct GemConfirmation {
    service: Arc<GemConfirmTransferService>,
    wallet: Wallet,
    transfer: Mutex<GemTransferData>,
    simulation: Option<SimulationResult>,
    state: Mutex<Option<ConfirmState>>,
    unanswered: Mutex<Option<SignedSubmission>>,
    latest_load: AtomicU64,
    created_at: Instant,
    authentication: GemKeystoreAuthentication,
}

impl GemConfirmation {
    pub(super) fn new(service: Arc<GemConfirmTransferService>, wallet: Wallet, transfer: GemTransferData, simulation: Option<SimulationResult>) -> Self {
        Self {
            authentication: service.authentication(),
            service,
            wallet,
            transfer: Mutex::new(transfer),
            simulation,
            state: Mutex::new(None),
            unanswered: Mutex::new(None),
            latest_load: AtomicU64::new(0),
            created_at: Instant::now(),
        }
    }

    fn stored(&self) -> MutexGuard<'_, Option<ConfirmState>> {
        self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn unanswered(&self) -> MutexGuard<'_, Option<SignedSubmission>> {
        self.unanswered.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn send_input(&self) -> Result<SendInput, GemConfirmError> {
        let Some(ConfirmState {
            load: GemConfirmLoad { fee: Some(fee), .. },
            confirm_data: Some(confirm_data),
            swap,
        }) = self.stored().clone()
        else {
            return Err(GemConfirmError::Load {
                msg: "confirm input is not loaded".to_string(),
            });
        };
        let amount = match fee.amount {
            GemTransferAmountResult::Amount { amount } => amount,
            GemTransferAmountResult::Error { error } => return Err(error),
        };
        Ok(SendInput {
            wallet: self.wallet.clone(),
            simulation: self.simulation.clone().or_else(|| confirm_data.simulation.clone()),
            confirm: confirm_data,
            value: amount.value,
            network_fee: amount.network_fee,
            swap_quote: swap.and_then(|swap| swap.quote),
        })
    }

    fn store_latest(&self, load: u64, result: Result<ConfirmState, GemConfirmError>) -> Result<GemConfirmLoad, GemConfirmError> {
        let mut stored = self.stored();
        if self.latest_load.load(Ordering::SeqCst) != load {
            return Err(GemConfirmError::Cancelled);
        }
        let state = result?;
        let loaded = state.load.clone();
        *self.transfer.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = loaded.transfer.clone();
        *stored = Some(state);
        Ok(loaded)
    }

    async fn load_screen(&self, options: &GemConfirmLoadOptions) -> Result<ConfirmState, GemConfirmError> {
        let now = Instant::now();
        let transfer = self.transfer();
        let held = self.stored().as_ref().and_then(|state| state.swap.clone()).or_else(|| ConfirmSwapQuote::initial(&transfer, self.created_at));
        let (transfer, swap) = Box::pin(self.service.refreshed_swap(&self.wallet, transfer, held, now)).await?;
        let input = self.service.confirm_input(self.wallet.clone(), transfer)?;
        let requested = async {
            match &self.simulation {
                Some(simulation) => Some(self.service.simulation_state(input.transfer.input_type.clone(), simulation.clone()).await),
                None => None,
            }
        };
        let (screen, fee, requested) = futures::join!(Box::pin(self.state()), Box::pin(self.load_fee(&input, options)), Box::pin(requested));
        let (fee, swap) = Box::pin(self.service.fitted_max_swap(&self.wallet, fee?, swap, now)).await?;
        let requested = requested.transpose()?;
        let screen = GemConfirmLoad {
            transfer: fee.confirm_data.input.transfer.clone(),
            ..screen?
        };
        Ok(screen.with_fee(fee, requested, swap))
    }

    fn simulation_warnings(&self, state: Option<&ConfirmState>) -> Vec<GemListRow> {
        match state {
            Some(state) => state.load.simulation.warnings.clone(),
            None => warning_rows(self.simulation.as_ref().map(|simulation| simulation.warnings.as_slice()).unwrap_or_default()),
        }
    }

    fn details(&self, transfer: &GemTransferData, load: Option<&GemConfirmLoad>) -> Option<GemConfirmDetails> {
        match &transfer.input_type {
            TransactionInputType::Swap { from_asset, to_asset, swap_data } => Some(GemConfirmDetails::Swap {
                details: swap_quote_details(
                    swap_data.quote.clone(),
                    from_asset.clone(),
                    to_asset.clone(),
                    load.and_then(|load| load.metadata.asset_price()).map(|price| price.price),
                    load.and_then(|load| load.metadata.price(to_asset.id.clone())).map(|price| price.price),
                    self.service.get_currency(),
                ),
            }),
            TransactionInputType::Perpetual {
                asset,
                perpetual_type: PerpetualType::Modify { data },
            } => autoclose_row(asset, data).map(|row| GemConfirmDetails::PerpetualAutoclose { row }),
            TransactionInputType::Perpetual { asset, perpetual_type } => perpetual_confirm_details(asset.clone(), perpetual_type.clone()).map(|details| GemConfirmDetails::Perpetual { details }),
            _ => None,
        }
    }

    async fn load_fee(&self, input: &GemConfirmInput, options: &GemConfirmLoadOptions) -> Result<GemConfirmFeeLoad, GemConfirmError> {
        let input_type = &input.transfer.input_type;
        let fee = match self.service.confirm().load(&self.wallet.id, input, options, self.service.get_currency()).await {
            Ok(fee) => fee,
            Err(error) => return Err(self.service.insufficient_funds(self.wallet.id.clone(), &input.transfer).await.unwrap_or(error)),
        };
        let simulation = match preload_simulation(self.simulation.as_ref(), &fee.confirm_data) {
            Some(simulation) => Some(self.service.simulation_state(input_type.clone(), simulation).await?),
            None => None,
        };
        Ok(GemConfirmFeeLoad { simulation, ..fee })
    }
}

#[uniffi::export]
impl GemConfirmation {
    pub fn screen(&self) -> GemConfirmScreen {
        GemConfirmScreen::initial(self.simulation.as_ref())
    }

    pub fn load_options(&self) -> GemConfirmLoadOptions {
        GemConfirmLoadOptions::initial(&self.transfer())
    }

    pub fn header(&self, screen: GemConfirmScreen) -> GemConfirmHeader {
        let transfer = self.transfer();
        let stored = self.stored();
        header::header(&transfer, self.simulation.as_ref(), stored.as_ref().map(|state| &state.load), self.service.get_currency(), &screen)
    }

    pub fn view_state(&self, screen: GemConfirmScreen) -> GemConfirmViewState {
        let state = self.stored().clone();
        let load = state.as_ref().map(|state| &state.load);
        let transfer = self.transfer();
        let send_started = screen.send_started() || self.unanswered().is_some();
        let rows = self
            .row_contents(load.and_then(|load| load.address_name.clone()))
            .into_iter()
            .map(|content| match content {
                GemConfirmRowContent::PaymentAsset { title, symbol, selectable, asset_ids } => GemConfirmRowContent::PaymentAsset {
                    title,
                    selectable: selectable && screen.phase != super::model::GemConfirmPhase::Loading && !send_started,
                    symbol,
                    asset_ids,
                },
                content => content,
            })
            .collect();
        let load_error = screen.failure.as_ref().filter(|failure| failure.stage == GemConfirmStage::Load).map(|failure| failure.error.clone());
        let verification = transfer.verification();
        let details = self.details(&transfer, load);
        GemConfirmViewState {
            button: screen.button().authenticated(self.authentication),
            details,
            fee_row: GemConfirmFeeRow::new(
                screen.fee_value(load.cloned()),
                load.map(|load| load.fee_asset.clone()).unwrap_or_else(|| transfer.fee_asset()),
                !send_started && state.as_ref().is_some_and(|state| state.confirm_data.is_some() || state.load.shows_fee_assets()),
            ),
            title: transfer.title(),
            sections: super::rules::confirm_sections(
                rows,
                transfer.payment_message(),
                self.simulation_warnings(state.as_ref()),
                load.and_then(|load| load.simulation.simulation.clone()),
                verification.is_some(),
                load_error,
            ),
            verification,
        }
    }

    pub fn network_fee_screen(&self, format: GemNumberFormat) -> Option<GemNetworkFeeScreen> {
        let stored = self.stored();
        Some(stored.as_ref()?.network_fee_screen(self.service.get_currency(), format))
    }

    pub fn row_contents(&self, address_name: Option<AddressName>) -> Vec<GemConfirmRowContent> {
        self.service.row_contents(self.transfer(), self.wallet.clone(), address_name)
    }

    pub fn error_info(&self, error: GemConfirmError) -> Option<GemConfirmErrorInfo> {
        let transfer = self.transfer();
        let (prices, fee_asset_id) = match self.stored().as_ref() {
            Some(state) => (state.load.metadata.prices.clone(), state.load.fee_asset.id.clone()),
            None => (Vec::new(), transfer.fee_asset().id),
        };
        super::error::confirm_error_info(error, prices, self.service.get_currency(), transfer.input_asset().id, fee_asset_id)
    }

    pub async fn submit(&self) -> Result<GemSubmitResult, GemConfirmError> {
        let held = self.unanswered().clone();
        let submission = match held {
            Some(submission) => submission,
            None => Box::pin(self.service.sign_submission(self.send_input()?)).await?,
        };
        match Box::pin(self.service.deliver(&submission)).await {
            Ok(result) => Ok(result),
            Err(SendFailure::Failed(error)) => Err(error),
            Err(SendFailure::Unanswered(error)) => {
                *self.unanswered() = Some(submission);
                Err(error)
            }
        }
    }

    pub fn transfer(&self) -> GemTransferData {
        self.transfer.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
    }

    pub async fn state(&self) -> Result<GemConfirmLoad, GemConfirmError> {
        if let Some(load) = self.stored().as_ref().map(|state| state.load.clone()) {
            return Ok(load);
        }
        let input = self.service.confirm_input(self.wallet.clone(), self.transfer())?;
        let load = self.service.state(self.wallet.id.clone(), &input, self.simulation.clone()).await?;
        Ok(self.stored().get_or_insert(ConfirmState { load, confirm_data: None, swap: None }).load.clone())
    }

    pub async fn load(&self, options: GemConfirmLoadOptions) -> Result<GemConfirmLoad, GemConfirmError> {
        let load = self.latest_load.fetch_add(1, Ordering::SeqCst) + 1;
        if self.unanswered().is_some() {
            return self.state().await;
        }
        if let Some(asset_id) = options.asset_id.clone() {
            self.select_asset(asset_id).await?;
        }
        if self.transfer().verification().is_some() {
            return self.state().await;
        }
        let result = Box::pin(self.load_screen(&options)).await;
        self.store_latest(load, result)
    }
}

impl GemConfirmation {
    async fn select_asset(&self, asset_id: AssetId) -> Result<(), GemConfirmError> {
        let transfer = self.transfer();
        if !asset_pick_needs_reload(&transfer.input_asset().id, &asset_id, transfer.verification().as_ref()) {
            return Ok(());
        }
        let TransactionInputType::Payment { invoice, .. } = transfer.input_type else {
            return Err(GemConfirmError::Load {
                msg: "Transfer is not a payment".to_string(),
            });
        };
        let addresses = self.wallet.accounts.iter().map(|account| ChainAddress::new(account.chain, account.address.clone())).collect();
        let transfer = match self.service.payment().select_asset(&invoice.link, addresses, asset_id).await? {
            GemPaymentLoad::Sign { transfer } => transfer,
            GemPaymentLoad::Verify { invoice, asset_id, url } => self.service.payment().quote_transfer_data(invoice, asset_id, PaymentVerification { url }).await?,
        };
        *self.transfer.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = transfer;
        *self.stored() = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use num_bigint::BigInt;
    use primitives::{Account, Asset, AssetId, Chain, FeePriority, PaymentInvoice, PaymentQuote, SimulationBalanceChange, SimulationResult, SimulationWarning, TransactionInputType, TransferDataExtra, Wallet, WalletId};

    use std::sync::atomic::Ordering;

    use primitives::{AddressName, AddressType, VerificationStatus};

    use super::super::error::GemConfirmErrorDisplay;
    use super::super::model::{GemConfirmAction, GemConfirmFeeValue, GemConfirmPhase, GemFeeRateKind};
    use super::super::testkit::{ConfirmTestkit, TEST_TRON_RECIPIENT, mock_bitcoin_transfer, mock_bitcoin_wallet, mock_tron_transfer, mock_tron_wallet};
    use super::GemConfirmation;
    use crate::models::button::GemButtonState;
    use crate::models::placeholder::EMPTY_VALUE;
    use crate::services::amount::model::GemNumberFormat;
    use crate::services::balance::GemBalanceRequirement;
    use crate::services::confirm::{
        ConfirmState, GemConfirmDetails, GemConfirmError, GemConfirmFee, GemConfirmFeeSelection, GemConfirmLoad, GemConfirmLoadOptions, GemConfirmRowContent, GemConfirmScreen, GemConfirmSection, GemConfirmViewState,
        GemTransferAmountResult, SendInput,
    };
    use crate::services::simulation::warning_rows;
    use crate::services::swap::model::swap_quote_details;
    use crate::services::transfer::{GemRecipient, GemTransferData};
    use crate::testkit::TestAlienProvider;
    use primitives::currency::Currency;
    use std::sync::Arc;

    #[test]
    fn test_request_wallet_balances_ignore_selected_wallet() {
        block_on(async {
            let wallet = mock_tron_wallet();
            let selected_wallet = Wallet {
                id: WalletId::Multicoin("other-wallet".into()),
                ..Wallet::mock()
            };
            let testkit = ConfirmTestkit::new(wallet.clone(), selected_wallet);
            let transfer = mock_tron_transfer(0.into());
            let confirmation = testkit.service.confirmation(wallet.clone(), transfer, None);

            let state = confirmation.state().await.unwrap();

            assert_eq!(state.sender.address, wallet.accounts[0].address);
            assert_eq!(state.sender.chain, Chain::Tron);
            assert_eq!(state.metadata.asset_balance.available, 100u32.into());
            assert_eq!(state.metadata.fee_asset_balance.available, 100u32.into());
            assert_eq!(*testkit.balances.requests.lock().unwrap(), vec![wallet.id.clone()]);

            let options = GemConfirmLoadOptions {
                fee_selection: GemConfirmFeeSelection::Priority { priority: FeePriority::Normal },
                fee_asset_id: None,
                asset_id: None,
            };
            assert!(confirmation.load(options).await.is_err());
            assert_eq!(*testkit.balances.requests.lock().unwrap(), vec![wallet.id.clone(), wallet.id]);
            assert!(testkit.balances.balance_writes.lock().unwrap().is_empty(), "confirming reads balances and never writes them");
            assert!(testkit.balances.enable_writes.lock().unwrap().is_empty());
        });
    }

    #[test]
    fn test_view_state_reads_one_screen() {
        let wallet = mock_tron_wallet();
        let testkit = ConfirmTestkit::new(wallet.clone(), wallet.clone());
        let transfer = mock_tron_transfer(10.into());
        let confirmation = testkit.service.confirmation(wallet, transfer, None);
        let screen = confirmation.screen();

        let state = confirmation.view_state(screen.clone());

        assert_eq!(state.button, screen.button().authenticated(confirmation.authentication));
        assert_eq!(state.fee_row, super::super::GemConfirmFeeRow::new(screen.fee_value(None), confirmation.transfer().fee_asset(), false));
        assert_eq!(confirmation.network_fee_screen(GemNumberFormat { decimal_separator: ".".to_string() }), None, "nothing loaded, no fee screen");
        assert_eq!(details(&state), confirmation.row_contents(None));
        assert_eq!(state.title, confirmation.transfer().title());
        assert_eq!(state.verification, None);
    }

    #[test]
    fn test_a_bitcoin_send_without_utxos_keeps_the_fee_options_above_its_balance_error() {
        block_on(async {
            let testkit = ConfirmTestkit::with_bitcoin_utxos(129_576, &[]);
            let confirmation = testkit.service.confirmation(mock_bitcoin_wallet(), mock_bitcoin_transfer(10_000.into()), None);

            let load = confirmation.load(confirmation.load_options()).await.unwrap();
            let fee = load.fee.clone().unwrap().value;
            let screen = confirmation.screen().on_loaded(load);
            let state = confirmation.view_state(screen.clone());
            let rates = confirmation.network_fee_screen(GemNumberFormat { decimal_separator: ".".to_string() }).and_then(|screen| screen.rates).unwrap();

            assert_eq!(screen.phase, GemConfirmPhase::Ready);
            assert_eq!(screen.button().state, GemButtonState::Disabled);
            assert!(state.fee_row.opens_details);
            assert_eq!(
                rates.rows.into_iter().map(|row| row.kind).collect::<Vec<_>>(),
                vec![GemFeeRateKind::Priority { priority: FeePriority::Normal }, GemFeeRateKind::Priority { priority: FeePriority::Fast }, GemFeeRateKind::Custom,],
            );
            assert_eq!(
                state.sections.last(),
                Some(&GemConfirmSection::Error {
                    error: GemConfirmError::InsufficientBalance {
                        asset: Asset::mock_btc(),
                        requirement: GemBalanceRequirement::new(BigInt::from(10_000) + fee, 0u64.into()),
                    },
                }),
            );
        });
    }

    #[test]
    fn test_a_fee_that_fails_the_load_can_be_changed_back() {
        block_on(async {
            let testkit = ConfirmTestkit::with_bitcoin_utxos(2_000, &[2_000]);
            let transfer = GemTransferData {
                use_max_amount: true,
                ..mock_bitcoin_transfer(2_000.into())
            };
            let confirmation = testkit.service.confirmation(mock_bitcoin_wallet(), transfer, None);
            let normal = confirmation.load_options();

            confirmation.load(normal.clone()).await.unwrap();
            let leaves_only_dust = GemConfirmLoadOptions {
                fee_selection: GemConfirmFeeSelection::Custom {
                    base_fee: None,
                    rate: Some(BigInt::from(170)),
                },
                ..normal.clone()
            };
            let error = confirmation.load(leaves_only_dust).await.unwrap_err();
            assert_eq!(error.display(), GemConfirmErrorDisplay::DustThreshold { chain: Chain::Bitcoin });
            let state = confirmation.view_state(confirmation.screen().on_load_failed(error));

            assert_eq!(state.fee_row.value, GemConfirmFeeValue::Unavailable { text: EMPTY_VALUE.to_string() });
            assert!(state.fee_row.opens_details, "the fee that failed is the one the user has to change");
            assert!(confirmation.network_fee_screen(GemNumberFormat { decimal_separator: ".".to_string() }).and_then(|screen| screen.rates).is_some());

            let load = confirmation.load(normal).await.unwrap();
            assert_eq!(confirmation.screen().on_loaded(load).action(), Some(GemConfirmAction::Execute));
        });
    }

    #[test]
    fn test_a_failed_first_load_has_no_fee_options_to_open() {
        block_on(async {
            let wallet = mock_tron_wallet();
            let testkit = ConfirmTestkit::new(wallet.clone(), wallet.clone());
            let confirmation = testkit.service.confirmation(wallet, mock_tron_transfer(10.into()), None);

            let error = confirmation.load(confirmation.load_options()).await.unwrap_err();
            let state = confirmation.view_state(confirmation.screen().on_load_failed(error));

            assert!(!state.fee_row.opens_details);
        });
    }

    #[test]
    fn test_a_sending_screen_keeps_the_fee_row_closed() {
        block_on(async {
            let testkit = ConfirmTestkit::with_bitcoin_utxos(100_000, &[100_000]);
            let confirmation = testkit.service.confirmation(mock_bitcoin_wallet(), mock_bitcoin_transfer(10_000.into()), None);
            let ready = confirmation.screen().on_loaded(confirmation.load(confirmation.load_options()).await.unwrap());
            let sending = ready.on_execute_started();

            assert!(confirmation.view_state(ready).fee_row.opens_details);
            assert!(!confirmation.view_state(sending.clone()).fee_row.opens_details, "a new fee would reload the screen under the transaction being sent");
            assert!(confirmation.view_state(sending.on_execute_failed(GemConfirmError::Offline)).fee_row.opens_details);
        });
    }

    #[test]
    fn test_a_sending_screen_keeps_the_payment_asset_closed() {
        let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Ethereum, "0x0000000000000000000000000000000000000001")]);
        let testkit = ConfirmTestkit::new(wallet.clone(), wallet.clone());
        let invoice = PaymentInvoice {
            quotes: vec![PaymentQuote::mock(AssetId::from_chain(Chain::Ethereum)), PaymentQuote::mock(AssetId::from_chain(Chain::SmartChain))],
            ..PaymentInvoice::mock()
        };
        let transfer = GemTransferData::mock(TransactionInputType::Payment {
            asset: Asset::mock_eth(),
            invoice,
            extra: TransferDataExtra::mock(),
        });
        let confirmation = testkit.service.confirmation(wallet, transfer, None);
        let selectable = |screen: GemConfirmScreen| {
            details(&confirmation.view_state(screen)).into_iter().find_map(|row| match row {
                GemConfirmRowContent::PaymentAsset { selectable, .. } => Some(selectable),
                _ => None,
            })
        };
        let ready = GemConfirmScreen {
            phase: GemConfirmPhase::Ready,
            ..GemConfirmScreen::initial(None)
        };
        let sending = ready.on_execute_started();

        assert_eq!(selectable(ready), Some(true));
        assert_eq!(selectable(sending.clone()), Some(false), "another asset would reload the screen under the payment being sent");
        assert_eq!(selectable(sending.on_execute_failed(GemConfirmError::Offline)), Some(true));
    }

    #[test]
    fn test_view_state_reads_the_authentication_once_per_screen() {
        let wallet = mock_tron_wallet();
        let testkit = ConfirmTestkit::new(wallet.clone(), wallet.clone());
        let transfer = mock_tron_transfer(10.into());
        let confirmation = testkit.service.confirmation(wallet, transfer, None);
        let screen = confirmation.screen();

        for _ in 0..3 {
            confirmation.view_state(screen.clone());
        }

        assert_eq!(testkit.passwords.authentication_reads.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_view_state_picks_the_details_block_by_input_type() {
        let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Ethereum, "0xsender")]);
        let testkit = ConfirmTestkit::new(wallet.clone(), wallet.clone());
        let (from_asset, to_asset) = (Asset::mock_eth(), Asset::from_chain(Chain::SmartChain));
        let swap_data = primitives::SwapData::mock();
        let swap = GemTransferData::mock(TransactionInputType::Swap {
            from_asset: from_asset.clone(),
            to_asset: to_asset.clone(),
            swap_data: swap_data.clone(),
        });
        let confirmation = testkit.service.clone().confirmation(wallet.clone(), swap, None);

        assert_eq!(
            confirmation.view_state(confirmation.screen()).details,
            Some(GemConfirmDetails::Swap {
                details: swap_quote_details(swap_data.quote, from_asset, to_asset, None, None, Currency::USD),
            }),
            "a swap shows its quote before the load prices it"
        );

        let transfer = testkit.service.confirmation(wallet, GemTransferData::mock(TransactionInputType::Transfer { asset: Asset::mock_eth() }), None);
        assert_eq!(transfer.view_state(transfer.screen()).details, None);
    }

    #[test]
    fn test_view_state_names_the_recipient_from_the_last_stored_load() {
        block_on(async {
            let wallet = mock_tron_wallet();
            let testkit = ConfirmTestkit::new(wallet.clone(), wallet.clone());
            let transfer = mock_tron_transfer(0.into());
            let confirmation = testkit.service.confirmation(wallet, transfer, None);
            let name = AddressName::mock(TEST_TRON_RECIPIENT, "Friend", AddressType::Address, VerificationStatus::Verified);
            let named = GemConfirmLoad {
                address_name: Some(name.clone()),
                ..confirmation.state().await.unwrap()
            };
            let load = confirmation.latest_load.fetch_add(1, Ordering::SeqCst) + 1;
            assert!(confirmation.store_latest(load, Ok(ConfirmState { load: named, confirm_data: None, swap: None })).is_ok());

            assert_eq!(details(&confirmation.view_state(confirmation.screen())), confirmation.row_contents(Some(name.clone())));

            let failed = confirmation.latest_load.fetch_add(1, Ordering::SeqCst) + 1;
            assert!(confirmation.store_latest(failed, Err(GemConfirmError::Offline)).is_err());
            assert_eq!(
                details(&confirmation.view_state(confirmation.screen())),
                confirmation.row_contents(Some(name)),
                "a failed reload keeps the name the screen already shows"
            );
        });
    }

    #[test]
    fn test_the_request_warnings_show_before_the_load() {
        let wallet = mock_tron_wallet();
        let testkit = ConfirmTestkit::new(wallet.clone(), wallet.clone());
        let transfer = mock_tron_transfer(10.into());
        let warnings = vec![SimulationWarning::validation_error("careful")];
        let simulation = SimulationResult {
            warnings: warnings.clone(),
            ..SimulationResult::default()
        };
        let confirmation = testkit.service.confirmation(wallet, transfer, Some(simulation));

        let state = confirmation.view_state(confirmation.screen());

        assert!(!warning_rows(&warnings).is_empty());
        assert!(state.sections.contains(&GemConfirmSection::Warnings { rows: warning_rows(&warnings) }));
    }

    fn details(state: &GemConfirmViewState) -> Vec<GemConfirmRowContent> {
        state
            .sections
            .iter()
            .find_map(|section| match section {
                GemConfirmSection::Details { rows } => Some(rows.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    #[test]
    fn test_a_load_overtaken_by_a_newer_one_is_cancelled_and_not_stored() {
        block_on(async {
            let wallet = mock_tron_wallet();
            let testkit = ConfirmTestkit::new(wallet.clone(), wallet.clone());
            let transfer = mock_tron_transfer(0.into());
            let confirmation = testkit.service.confirmation(wallet, transfer, None);
            let shown = confirmation.state().await.unwrap();
            let stale = GemConfirmLoad {
                address_name: Some(AddressName::mock(TEST_TRON_RECIPIENT, "stale", AddressType::Address, VerificationStatus::Unverified)),
                ..shown
            };

            let older = confirmation.latest_load.fetch_add(1, Ordering::SeqCst) + 1;
            let newer = confirmation.latest_load.fetch_add(1, Ordering::SeqCst) + 1;

            let state = |load: GemConfirmLoad| Ok(ConfirmState { load, confirm_data: None, swap: None });
            assert_eq!(confirmation.store_latest(older, state(stale.clone())).unwrap_err(), GemConfirmError::Cancelled);
            assert_eq!(confirmation.store_latest(older, Err(GemConfirmError::Offline)).unwrap_err(), GemConfirmError::Cancelled);
            assert!(confirmation.state().await.unwrap().address_name.is_none());

            assert!(confirmation.store_latest(newer, state(stale)).is_ok());
            assert!(confirmation.state().await.unwrap().address_name.is_some());
        });
    }

    #[test]
    fn test_execute_refuses_a_confirmation_without_a_preload() {
        block_on(async {
            let wallet = mock_tron_wallet();
            let testkit = ConfirmTestkit::new(wallet.clone(), wallet.clone());
            let transfer = mock_tron_transfer(0.into());
            let confirmation = testkit.service.confirmation(wallet, transfer, None);

            assert!(matches!(confirmation.submit().await, Err(GemConfirmError::Load { .. })));
            confirmation.state().await.unwrap();
            assert!(matches!(confirmation.submit().await, Err(GemConfirmError::Load { .. })));
        });
    }

    #[test]
    fn test_state_shows_the_request_simulation_without_waiting_for_its_assets_and_names() {
        block_on(async {
            let wallet = mock_tron_wallet();
            let testkit = ConfirmTestkit::new(wallet.clone(), wallet.clone());
            let simulation = SimulationResult {
                warnings: vec![SimulationWarning::validation_error("careful")],
                balance_changes: vec![SimulationBalanceChange::mock(AssetId::from(Chain::Tron, Some("unknown-token".into())), BigInt::from(1), 6)],
                ..SimulationResult::default()
            };
            let transfer = mock_tron_transfer(0.into());
            let confirmation = testkit.service.confirmation(wallet, transfer, Some(simulation.clone()));

            let state = confirmation.state().await.unwrap();

            assert_eq!(state.simulation.warnings.len(), 1);
            assert!(state.simulation.simulation.is_none());
        });
    }

    #[test]
    fn test_select_asset() {
        block_on(async {
            let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Ethereum, "0x0000000000000000000000000000000000000001")]);
            let testkit = ConfirmTestkit::new(wallet.clone(), wallet.clone());
            let confirmation = |input_type| testkit.service.clone().confirmation(wallet.clone(), GemTransferData::mock(input_type), None);
            let ethereum = Asset::from_chain(Chain::Ethereum);
            let sent = confirmation(TransactionInputType::Transfer { asset: ethereum.clone() });
            let paid = confirmation(TransactionInputType::mock_payment(ethereum.clone(), TransferDataExtra::mock()));

            assert_eq!(sent.select_asset(AssetId::from_chain(Chain::SmartChain)).await.map_err(|error| error.to_string()), Err("Transfer is not a payment".to_string()));
            assert_eq!(paid.select_asset(ethereum.id).await.map_err(|error| error.to_string()), Ok(()), "the asset already paid with is not selected again");
        });
    }

    fn unanswered_send() -> (ConfirmTestkit, Arc<TestAlienProvider>, Arc<GemConfirmation>) {
        let provider = Arc::new(TestAlienProvider::offline());
        let testkit = ConfirmTestkit::with_provider(Wallet::mock_with_chains(&[Chain::Ethereum]), Wallet::mock_with_chains(&[Chain::Ethereum]), provider.clone());
        let wallet = testkit.keystore_wallet();
        let transfer = GemTransferData {
            recipient: GemRecipient::address(wallet.accounts[0].address.clone()),
            ..GemTransferData::mock(TransactionInputType::Transfer { asset: Asset::mock_eth() })
        };
        let input = SendInput::mock_signed_by(wallet.clone(), transfer.clone(), None);
        let confirmation = testkit.service.clone().confirmation(wallet, transfer, None);
        *confirmation.stored() = Some(ConfirmState {
            load: GemConfirmLoad {
                fee: Some(GemConfirmFee::mock(GemTransferAmountResult::mock())),
                ..GemConfirmLoad::mock()
            },
            confirm_data: Some(input.confirm),
            swap: None,
        });
        (testkit, provider, confirmation)
    }

    #[test]
    fn test_a_send_the_network_never_answered_goes_out_again_without_signing() {
        block_on(async {
            let (testkit, provider, confirmation) = unanswered_send();

            assert_eq!(confirmation.submit().await.unwrap_err(), GemConfirmError::Offline);
            assert_eq!(confirmation.submit().await.unwrap_err(), GemConfirmError::Offline);

            assert_eq!(testkit.passwords.create_requests.lock().unwrap().clone(), vec![false], "the second Confirm sends what the first one signed");
            let broadcasts = provider.requested_paths();
            assert_eq!(broadcasts.len(), 2);
            assert_eq!(broadcasts[0], broadcasts[1]);
        });
    }

    #[test]
    fn test_a_send_the_network_never_answered_keeps_the_fee_it_was_signed_with() {
        block_on(async {
            let (_testkit, provider, confirmation) = unanswered_send();
            assert_eq!(confirmation.submit().await.unwrap_err(), GemConfirmError::Offline);
            let requests = provider.requested_paths().len();

            let held = confirmation.load(confirmation.load_options()).await.unwrap();
            let failed = GemConfirmScreen::initial(None).on_loaded(held.clone()).on_execute_started().on_execute_failed(GemConfirmError::Offline);

            assert_eq!(provider.requested_paths().len(), requests, "a reload asks the network for nothing");
            assert_eq!(held.fee.map(|fee| fee.value), Some(BigInt::from(1)));
            assert!(!confirmation.view_state(failed).fee_row.opens_details, "the fee cannot change under a signed transaction");
        });
    }
}
