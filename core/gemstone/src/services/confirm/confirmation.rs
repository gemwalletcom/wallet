use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use primitives::currency::Currency;
use primitives::{AddressName, AssetId, ChainAddress, PaymentVerification, PerpetualModifyConfirmData, SimulationResult, TransactionInputType, Wallet};

use super::error::GemConfirmErrorInfo;
use super::header::{self, GemConfirmHeader};
use super::rules::{asset_pick_needs_reload, preload_simulation};
use super::{
    ConfirmState, GemConfirmError, GemConfirmFeeLoad, GemConfirmInput, GemConfirmLoad, GemConfirmLoadOptions, GemConfirmRowContent, GemConfirmScreen, GemConfirmTransferService, GemConfirmViewState, GemFeeRateRows, GemSubmitResult,
    GemTransferAmountResult, SendInput,
};
use crate::models::list::GemListRow;
use crate::payment::GemPaymentLoad;
use crate::services::simulation::warning_rows;
use crate::services::transfer::GemTransferData;
use crate::services::wallet::GemKeystoreAuthentication;

#[derive(uniffi::Object)]
pub struct GemConfirmation {
    service: Arc<GemConfirmTransferService>,
    wallet: Wallet,
    transfer: Mutex<GemTransferData>,
    simulation: Option<SimulationResult>,
    state: Mutex<Option<ConfirmState>>,
    latest_load: AtomicU64,
}

impl GemConfirmation {
    pub(super) fn new(service: Arc<GemConfirmTransferService>, wallet: Wallet, transfer: GemTransferData, simulation: Option<SimulationResult>) -> Self {
        Self {
            service,
            wallet,
            transfer: Mutex::new(transfer),
            simulation,
            state: Mutex::new(None),
            latest_load: AtomicU64::new(0),
        }
    }

    fn stored(&self) -> MutexGuard<'_, Option<ConfirmState>> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn store_latest(&self, load: u64, result: Result<ConfirmState, GemConfirmError>) -> Result<GemConfirmLoad, GemConfirmError> {
        let mut stored = self.stored();
        if self.latest_load.load(Ordering::SeqCst) != load {
            return Err(GemConfirmError::Cancelled);
        }
        let state = result?;
        let loaded = state.load.clone();
        *stored = Some(state);
        Ok(loaded)
    }

    async fn load_screen(&self, options: &GemConfirmLoadOptions) -> Result<ConfirmState, GemConfirmError> {
        let input = self.service.confirm_input(self.wallet.clone(), self.transfer())?;
        let requested = async {
            match &self.simulation {
                Some(simulation) => Some(self.service.simulation_state(input.transfer.input_type.clone(), simulation.clone()).await),
                None => None,
            }
        };
        let (screen, fee, requested) = futures::join!(Box::pin(self.state()), Box::pin(self.load_fee(&input, options)), Box::pin(requested));
        let fee = fee?;
        let requested = requested.transpose()?;
        Ok(screen?.with_fee(fee, requested))
    }

    fn authentication(&self) -> GemKeystoreAuthentication {
        self.service.authentication()
    }

    fn simulation_warnings(&self) -> Vec<GemListRow> {
        match self.stored().as_ref() {
            Some(state) => state.load.simulation.warnings.clone(),
            None => warning_rows(self.simulation.as_ref().map(|simulation| simulation.warnings.as_slice()).unwrap_or_default()),
        }
    }

    async fn load_fee(&self, input: &GemConfirmInput, options: &GemConfirmLoadOptions) -> Result<GemConfirmFeeLoad, GemConfirmError> {
        let input_type = &input.transfer.input_type;
        let fee = match self.service.confirm().load(&self.wallet.id, input, options).await {
            Ok(fee) => fee,
            Err(error) => return Err(self.service.missing_network_fee(self.wallet.id.clone(), input_type.clone()).await.unwrap_or(error)),
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
        let address_name = self.stored().as_ref().and_then(|state| state.load.address_name.clone());
        let transfer = self.transfer();
        GemConfirmViewState {
            button: screen.button(),
            fee_row: screen.fee_row(),
            fee_rates: self.fee_rate_rows(),
            row_contents: self
                .row_contents(address_name)
                .into_iter()
                .map(|content| match content {
                    GemConfirmRowContent::PaymentAsset { symbol, selectable, asset_ids } => GemConfirmRowContent::PaymentAsset {
                        selectable: selectable && screen.phase != super::model::GemConfirmPhase::Loading,
                        symbol,
                        asset_ids,
                    },
                    content => content,
                })
                .collect(),
            simulation_warnings: self.simulation_warnings(),
            title: transfer.title(),
            verification: transfer.verification(),
            authentication: self.authentication(),
        }
    }

    pub fn fee_rate_rows(&self) -> Option<GemFeeRateRows> {
        let stored = self.stored();
        let state = stored.as_ref()?;
        Some(state.confirm_data.as_ref()?.fee_rate_rows(&state.load.fee_asset))
    }

    pub fn get_currency(&self) -> Currency {
        self.service.get_currency()
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
        super::error::confirm_error_info(error, prices, self.get_currency(), transfer.input_asset().id, fee_asset_id)
    }

    pub fn autoclose_row(&self, data: PerpetualModifyConfirmData) -> Option<GemListRow> {
        self.service.autoclose_row(data)
    }

    pub async fn submit(&self) -> Result<GemSubmitResult, GemConfirmError> {
        let Some(ConfirmState {
            load: GemConfirmLoad { fee: Some(fee), .. },
            confirm_data: Some(confirm_data),
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
        let input = SendInput {
            wallet: self.wallet.clone(),
            simulation: self.simulation.clone().or_else(|| confirm_data.simulation.clone()),
            confirm: confirm_data,
            value: amount.value,
            network_fee: amount.network_fee,
        };
        self.service.submit(input).await
    }

    pub fn transfer(&self) -> GemTransferData {
        self.transfer.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clone()
    }

    pub async fn state(&self) -> Result<GemConfirmLoad, GemConfirmError> {
        if let Some(load) = self.stored().as_ref().map(|state| state.load.clone()) {
            return Ok(load);
        }
        let input = self.service.confirm_input(self.wallet.clone(), self.transfer())?;
        let load = self.service.state(self.wallet.id.clone(), &input, self.simulation.clone()).await?;
        Ok(self.stored().get_or_insert(ConfirmState { load, confirm_data: None }).load.clone())
    }

    pub async fn load(&self, options: GemConfirmLoadOptions) -> Result<GemConfirmLoad, GemConfirmError> {
        let load = self.latest_load.fetch_add(1, Ordering::SeqCst) + 1;
        if let Some(asset_id) = options.asset_id.clone() {
            self.select_asset(asset_id).await?;
        }
        if self.transfer().verification().is_some() {
            return self.state().await;
        }
        let result = self.load_screen(&options).await;
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
        *self.transfer.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = transfer;
        *self.stored() = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use num_bigint::BigInt;
    use primitives::{Account, Asset, AssetId, Chain, FeePriority, SimulationBalanceChange, SimulationResult, SimulationWarning, TransactionInputType, TransferDataExtra, Wallet, WalletId};

    use std::sync::atomic::Ordering;

    use primitives::{AddressName, AddressType, VerificationStatus};

    use super::super::testkit::ConfirmTestkit;
    use crate::services::confirm::{ConfirmState, GemConfirmError, GemConfirmFeeSelection, GemConfirmLoad, GemConfirmLoadOptions};
    use crate::services::simulation::warning_rows;
    use crate::services::transfer::{GemRecipient, GemTransferData};

    #[test]
    fn test_request_wallet_balances_ignore_selected_wallet() {
        block_on(async {
            let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Tron, "TJRyWwFs9wTFGZg3JbrVriFbNfCug5tDeC")]);
            let selected_wallet = Wallet {
                id: WalletId::Multicoin("other-wallet".into()),
                ..Wallet::mock()
            };
            let testkit = ConfirmTestkit::new(wallet.clone(), selected_wallet);
            let transfer = GemTransferData {
                recipient: GemRecipient::address("THTR75o8xXAgCTQqpiot2AFRAjvW1tSbVV".into()),
                value: 0.into(),
                ..GemTransferData::mock(TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Tron) })
            };
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
        let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Tron, "TJRyWwFs9wTFGZg3JbrVriFbNfCug5tDeC")]);
        let testkit = ConfirmTestkit::new(wallet.clone(), wallet.clone());
        let transfer = GemTransferData {
            recipient: GemRecipient::address("THTR75o8xXAgCTQqpiot2AFRAjvW1tSbVV".into()),
            ..GemTransferData::mock(TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Tron) })
        };
        let confirmation = testkit.service.confirmation(wallet, transfer, None);
        let screen = confirmation.screen();

        let state = confirmation.view_state(screen.clone());

        assert_eq!(state.button, screen.button());
        assert_eq!(state.fee_row, screen.fee_row());
        assert_eq!(state.fee_rates, confirmation.fee_rate_rows());
        assert_eq!(state.row_contents, confirmation.row_contents(None));
        assert_eq!(state.title, confirmation.transfer().title());
        assert_eq!(state.verification, None);
        assert_eq!(state.authentication, confirmation.authentication());
    }

    #[test]
    fn test_view_state_names_the_recipient_from_the_last_stored_load() {
        block_on(async {
            let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Tron, "TJRyWwFs9wTFGZg3JbrVriFbNfCug5tDeC")]);
            let testkit = ConfirmTestkit::new(wallet.clone(), wallet.clone());
            let transfer = GemTransferData {
                recipient: GemRecipient::address("THTR75o8xXAgCTQqpiot2AFRAjvW1tSbVV".into()),
                value: 0.into(),
                ..GemTransferData::mock(TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Tron) })
            };
            let confirmation = testkit.service.confirmation(wallet, transfer, None);
            let name = AddressName::mock("THTR75o8xXAgCTQqpiot2AFRAjvW1tSbVV", "Friend", AddressType::Address, VerificationStatus::Verified);
            let named = GemConfirmLoad {
                address_name: Some(name.clone()),
                ..confirmation.state().await.unwrap()
            };
            let load = confirmation.latest_load.fetch_add(1, Ordering::SeqCst) + 1;
            assert!(confirmation.store_latest(load, Ok(ConfirmState { load: named, confirm_data: None })).is_ok());

            assert_eq!(confirmation.view_state(confirmation.screen()).row_contents, confirmation.row_contents(Some(name.clone())));

            let failed = confirmation.latest_load.fetch_add(1, Ordering::SeqCst) + 1;
            assert!(confirmation.store_latest(failed, Err(GemConfirmError::Offline)).is_err());
            assert_eq!(
                confirmation.view_state(confirmation.screen()).row_contents,
                confirmation.row_contents(Some(name)),
                "a failed reload keeps the name the screen already shows"
            );
        });
    }

    #[test]
    fn test_the_request_warnings_show_before_the_load() {
        let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Tron, "TJRyWwFs9wTFGZg3JbrVriFbNfCug5tDeC")]);
        let testkit = ConfirmTestkit::new(wallet.clone(), wallet.clone());
        let transfer = GemTransferData {
            recipient: GemRecipient::address("THTR75o8xXAgCTQqpiot2AFRAjvW1tSbVV".into()),
            ..GemTransferData::mock(TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Tron) })
        };
        let warnings = vec![SimulationWarning::validation_error("careful")];
        let simulation = SimulationResult {
            warnings: warnings.clone(),
            ..SimulationResult::default()
        };
        let confirmation = testkit.service.confirmation(wallet, transfer, Some(simulation));

        let state = confirmation.view_state(confirmation.screen());

        assert_eq!(state.simulation_warnings, warning_rows(&warnings));
        assert!(!state.simulation_warnings.is_empty());
    }

    #[test]
    fn test_a_load_overtaken_by_a_newer_one_is_cancelled_and_not_stored() {
        block_on(async {
            let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Tron, "TJRyWwFs9wTFGZg3JbrVriFbNfCug5tDeC")]);
            let testkit = ConfirmTestkit::new(wallet.clone(), wallet.clone());
            let transfer = GemTransferData {
                recipient: GemRecipient::address("THTR75o8xXAgCTQqpiot2AFRAjvW1tSbVV".into()),
                value: 0.into(),
                ..GemTransferData::mock(TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Tron) })
            };
            let confirmation = testkit.service.confirmation(wallet, transfer, None);
            let shown = confirmation.state().await.unwrap();
            let stale = GemConfirmLoad {
                address_name: Some(AddressName::mock("THTR75o8xXAgCTQqpiot2AFRAjvW1tSbVV", "stale", AddressType::Address, VerificationStatus::Unverified)),
                ..shown
            };

            let older = confirmation.latest_load.fetch_add(1, Ordering::SeqCst) + 1;
            let newer = confirmation.latest_load.fetch_add(1, Ordering::SeqCst) + 1;

            let state = |load: GemConfirmLoad| Ok(ConfirmState { load, confirm_data: None });
            assert!(matches!(confirmation.store_latest(older, state(stale.clone())), Err(GemConfirmError::Cancelled)));
            assert!(matches!(confirmation.store_latest(older, Err(GemConfirmError::Offline)), Err(GemConfirmError::Cancelled)));
            assert!(confirmation.state().await.unwrap().address_name.is_none());

            assert!(confirmation.store_latest(newer, state(stale)).is_ok());
            assert!(confirmation.state().await.unwrap().address_name.is_some());
        });
    }

    #[test]
    fn test_execute_refuses_a_confirmation_without_a_preload() {
        block_on(async {
            let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Tron, "TJRyWwFs9wTFGZg3JbrVriFbNfCug5tDeC")]);
            let testkit = ConfirmTestkit::new(wallet.clone(), wallet.clone());
            let transfer = GemTransferData {
                recipient: GemRecipient::address("THTR75o8xXAgCTQqpiot2AFRAjvW1tSbVV".into()),
                value: 0.into(),
                ..GemTransferData::mock(TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Tron) })
            };
            let confirmation = testkit.service.confirmation(wallet, transfer, None);

            assert!(matches!(confirmation.submit().await, Err(GemConfirmError::Load { .. })));
            confirmation.state().await.unwrap();
            assert!(matches!(confirmation.submit().await, Err(GemConfirmError::Load { .. })));
        });
    }

    #[test]
    fn test_state_shows_the_request_simulation_without_waiting_for_its_assets_and_names() {
        block_on(async {
            let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Tron, "TJRyWwFs9wTFGZg3JbrVriFbNfCug5tDeC")]);
            let testkit = ConfirmTestkit::new(wallet.clone(), wallet.clone());
            let simulation = SimulationResult {
                warnings: vec![SimulationWarning::validation_error("careful")],
                balance_changes: vec![SimulationBalanceChange::mock(AssetId::from(Chain::Tron, Some("unknown-token".into())), BigInt::from(1), 6)],
                ..SimulationResult::default()
            };
            let transfer = GemTransferData {
                recipient: GemRecipient::address("THTR75o8xXAgCTQqpiot2AFRAjvW1tSbVV".into()),
                value: 0.into(),
                ..GemTransferData::mock(TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Tron) })
            };
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
}
