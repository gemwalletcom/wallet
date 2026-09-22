use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use futures::lock::Mutex as AsyncMutex;
use primitives::currency::Currency;
use primitives::{AddressName, AssetId, BlockExplorerLink, Chain, ChainAddress, PaymentVerification, PerpetualModifyConfirmData, SimulationResult, TransactionInputType, Wallet};

use super::error::GemConfirmErrorInfo;
use super::header::{self, GemConfirmHeader};
use super::model::GemConfirmMetadata;
use super::rules::{acquire_swap_pair, preload_simulation};
use super::{GemAcquireAssetFlow, GemConfirmError, GemConfirmLoad, GemConfirmLoadOptions, GemConfirmRowContent, GemConfirmScreen, GemConfirmTransferService, GemSubmitResult, GemTransferAmountResult};
use crate::models::list::GemListRow;
use crate::payment::GemPaymentLoad;
use crate::services::swap::model::GemSwapPairSelection;
use crate::services::transfer::GemTransferData;
use crate::services::wallet::GemKeystoreAuthentication;

#[derive(uniffi::Object)]
pub struct GemConfirmation {
    service: Arc<GemConfirmTransferService>,
    wallet: Wallet,
    transfer: Mutex<GemTransferData>,
    simulation: Option<SimulationResult>,
    screen: AsyncMutex<Option<GemConfirmLoad>>,
    latest_load: AtomicU64,
}

impl GemConfirmation {
    pub(super) fn new(service: Arc<GemConfirmTransferService>, wallet: Wallet, transfer: GemTransferData, simulation: Option<SimulationResult>) -> Self {
        Self {
            service,
            wallet,
            transfer: Mutex::new(transfer),
            simulation,
            screen: AsyncMutex::new(None),
            latest_load: AtomicU64::new(0),
        }
    }

    async fn store_latest(&self, load: u64, result: Result<GemConfirmLoad, GemConfirmError>) -> Result<GemConfirmLoad, GemConfirmError> {
        let mut stored = self.screen.lock().await;
        if self.latest_load.load(Ordering::SeqCst) != load {
            return Err(GemConfirmError::Cancelled);
        }
        let screen = result?;
        *stored = Some(screen.clone());
        Ok(screen)
    }

    async fn load_screen(&self, options: GemConfirmLoadOptions) -> Result<GemConfirmLoad, GemConfirmError> {
        let transfer = self.transfer();
        let input = self.service.confirm_input(self.wallet.clone(), transfer.clone())?;
        let input_type = transfer.input_type;
        let requested = async {
            match self.simulation.clone() {
                Some(simulation) => Some(self.service.simulation_state(input_type.clone(), Some(simulation)).await),
                None => None,
            }
        };
        let (screen, fee, requested) = futures::join!(self.state(), self.service.preload(self.wallet.id.clone(), input, options), requested);
        let fee = fee?;
        let simulation = match preload_simulation(self.simulation.as_ref(), &fee.preload) {
            Some(simulation) => Some(self.service.simulation_state(input_type, Some(simulation)).await?),
            None => requested.transpose()?,
        };
        Ok(screen?.with_fee(fee, simulation))
    }
}

#[uniffi::export]
impl GemConfirmation {
    pub fn screen(&self) -> GemConfirmScreen {
        GemConfirmScreen::initial(self.simulation.as_ref())
    }

    pub fn header(&self, load: Option<GemConfirmLoad>) -> GemConfirmHeader {
        header::header(&self.transfer(), self.simulation.as_ref(), load.as_ref(), self.service.get_currency())
    }

    pub fn get_currency(&self) -> Currency {
        self.service.get_currency()
    }

    pub fn authentication(&self) -> GemKeystoreAuthentication {
        self.service.authentication()
    }

    pub fn row_contents(&self, address_name: Option<AddressName>) -> Vec<GemConfirmRowContent> {
        self.service.row_contents(self.transfer(), self.wallet.clone(), address_name)
    }

    pub fn address_url(&self, chain: Chain, address: String) -> BlockExplorerLink {
        self.service.address_url(chain, address)
    }

    pub fn acquire_asset_flow(&self, chain: Chain) -> GemAcquireAssetFlow {
        self.service.acquire_asset_flow(chain)
    }

    pub fn acquire_swap_pair(&self, fee_asset_id: Option<AssetId>, asset_id: AssetId) -> GemSwapPairSelection {
        let transfer = self.transfer();
        let fee_asset_id = fee_asset_id.unwrap_or_else(|| transfer.fee_asset().id);
        acquire_swap_pair(&transfer.input_asset().id, &fee_asset_id, asset_id)
    }

    pub fn error_info(&self, error: GemConfirmError, metadata: Option<GemConfirmMetadata>) -> Option<GemConfirmErrorInfo> {
        super::error::confirm_error_info(error, metadata.map(|metadata| metadata.prices).unwrap_or_default(), self.get_currency())
    }

    pub fn insufficient_network_fee_buy_amount(&self) -> i32 {
        self.service.insufficient_network_fee_buy_amount()
    }

    pub fn autoclose_row(&self, data: PerpetualModifyConfirmData) -> Option<GemListRow> {
        self.service.autoclose_row(data)
    }

    pub async fn submit(&self) -> Result<GemSubmitResult, GemConfirmError> {
        let screen = self.screen.lock().await.clone();
        let preload = screen.as_ref().and_then(|screen| screen.preload.clone()).ok_or_else(|| GemConfirmError::Load {
            msg: "confirm input is not loaded".to_string(),
        })?;
        let amount = match preload.amount {
            GemTransferAmountResult::Amount { amount } => amount,
            GemTransferAmountResult::Error { error } => return Err(error),
        };
        let simulation = screen.and_then(|screen| screen.simulation.result);
        self.service.submit(self.wallet.clone(), preload.confirm_data, amount.value, amount.network_fee, simulation).await
    }

    pub fn transfer(&self) -> GemTransferData {
        self.transfer.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clone()
    }

    pub async fn state(&self) -> Result<GemConfirmLoad, GemConfirmError> {
        if let Some(screen) = self.screen.lock().await.clone() {
            return Ok(screen);
        }
        let input = self.service.confirm_input(self.wallet.clone(), self.transfer())?;
        let screen = self.service.state(self.wallet.id.clone(), &input, self.simulation.clone()).await?;
        *self.screen.lock().await = Some(screen.clone());
        Ok(screen)
    }

    pub async fn load(&self, options: GemConfirmLoadOptions) -> Result<GemConfirmLoad, GemConfirmError> {
        let load = self.latest_load.fetch_add(1, Ordering::SeqCst) + 1;
        if let Some(asset_id) = options.asset_id.clone() {
            self.select_asset(asset_id).await?;
        }
        if self.transfer().verification().is_some() {
            return self.state().await;
        }
        let result = self.load_screen(options).await;
        self.store_latest(load, result).await
    }
}

impl GemConfirmation {
    async fn select_asset(&self, asset_id: AssetId) -> Result<(), GemConfirmError> {
        let transfer = self.transfer();
        if transfer.input_type.get_asset().id == asset_id && transfer.verification().is_none() {
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
        *self.screen.lock().await = None;
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
    use crate::services::confirm::{GemConfirmError, GemConfirmFeeSelection, GemConfirmLoad, GemConfirmLoadOptions};
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

            assert!(matches!(confirmation.store_latest(older, Ok(stale.clone())).await, Err(GemConfirmError::Cancelled)));
            assert!(matches!(confirmation.store_latest(older, Err(GemConfirmError::Offline)).await, Err(GemConfirmError::Cancelled)));
            assert!(confirmation.state().await.unwrap().address_name.is_none());

            assert!(confirmation.store_latest(newer, Ok(stale)).await.is_ok());
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

            assert_eq!(state.simulation.result, Some(simulation));
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
