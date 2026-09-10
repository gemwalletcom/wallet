use std::sync::Arc;

use futures::lock::Mutex;
use primitives::currency::Currency;
use primitives::{AssetId, BlockExplorerLink, Chain, ChainAddress, PerpetualModifyConfirmData, SimulationResult, TransactionInputType, Wallet};

use super::rules::preload_simulation;
use super::{GemAcquireAssetFlow, GemConfirmError, GemConfirmLoad, GemConfirmLoadOptions, GemConfirmScreen, GemConfirmTransferService, GemExecuteResult, GemTransferAmountResult};
use crate::payment::GemPaymentLoad;
use crate::services::perpetual::model::GemAutocloseSummary;
use crate::services::transfer::GemTransferData;
use crate::services::wallet::GemKeystoreAuthentication;

#[derive(uniffi::Object)]
pub struct GemConfirmSession {
    service: Arc<GemConfirmTransferService>,
    wallet: Wallet,
    transfer: Mutex<GemTransferData>,
    simulation: Option<SimulationResult>,
    screen: Mutex<Option<GemConfirmLoad>>,
}

impl GemConfirmSession {
    pub(super) fn new(service: Arc<GemConfirmTransferService>, wallet: Wallet, transfer: GemTransferData, simulation: Option<SimulationResult>) -> Self {
        Self {
            service,
            wallet,
            transfer: Mutex::new(transfer),
            simulation,
            screen: Mutex::new(None),
        }
    }
}

#[uniffi::export]
impl GemConfirmSession {
    pub fn screen(&self) -> GemConfirmScreen {
        GemConfirmScreen::initial(self.simulation.as_ref())
    }

    pub fn get_currency(&self) -> Currency {
        self.service.get_currency()
    }

    pub fn authentication(&self) -> GemKeystoreAuthentication {
        self.service.authentication()
    }

    pub fn address_url(&self, chain: Chain, address: String) -> BlockExplorerLink {
        self.service.address_url(chain, address)
    }

    pub fn acquire_asset_flow(&self, chain: Chain) -> GemAcquireAssetFlow {
        self.service.acquire_asset_flow(chain)
    }

    pub fn insufficient_network_fee_buy_amount(&self) -> i32 {
        self.service.insufficient_network_fee_buy_amount()
    }

    pub fn autoclose_summary(&self, data: PerpetualModifyConfirmData) -> Option<GemAutocloseSummary> {
        self.service.autoclose_summary(data)
    }

    pub async fn execute(&self) -> Result<GemExecuteResult, GemConfirmError> {
        let screen = self.screen.lock().await.clone();
        let preload = screen.as_ref().and_then(|screen| screen.preload.clone()).ok_or_else(|| GemConfirmError::Load {
            msg: "confirm input is not loaded".to_string(),
        })?;
        let amount = match preload.amount {
            GemTransferAmountResult::Amount { amount } => amount,
            GemTransferAmountResult::Error { error } => return Err(error),
        };
        let simulation = screen.and_then(|screen| screen.simulation.result);
        self.service
            .execute(self.wallet.clone(), preload.confirm_data, amount.value, amount.network_fee, simulation)
            .await
    }

    pub async fn state(&self) -> Result<GemConfirmLoad, GemConfirmError> {
        if let Some(screen) = self.screen.lock().await.clone() {
            return Ok(screen);
        }
        let input = self.service.confirm_input(self.wallet.clone(), self.transfer.lock().await.clone())?;
        let screen = self.service.state(self.wallet.id.clone(), &input, self.simulation.clone()).await?;
        *self.screen.lock().await = Some(screen.clone());
        Ok(screen)
    }

    pub async fn load(&self, options: GemConfirmLoadOptions) -> Result<GemConfirmLoad, GemConfirmError> {
        if let Some(asset_id) = options.asset_id.clone() {
            self.select_asset(asset_id).await?;
        }
        let transfer = self.transfer.lock().await.clone();
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
        let screen = screen?.with_fee(fee, simulation);
        *self.screen.lock().await = Some(screen.clone());
        Ok(screen)
    }
}

impl GemConfirmSession {
    async fn select_asset(&self, asset_id: AssetId) -> Result<(), GemConfirmError> {
        let transfer = self.transfer.lock().await.clone();
        if transfer.input_type.get_asset().id == asset_id {
            return Ok(());
        }
        let TransactionInputType::Payment { invoice, .. } = transfer.input_type else {
            return Err(GemConfirmError::Load {
                msg: "Transfer is not a payment".to_string(),
            });
        };
        let addresses = self.wallet.accounts.iter().map(|account| ChainAddress::new(account.chain, account.address.clone())).collect();
        let transfer = match self.service.payment().select_asset(invoice, addresses, asset_id).await? {
            GemPaymentLoad::Sign { transfer } => transfer,
            GemPaymentLoad::Verify { .. } => {
                return Err(GemConfirmError::Load {
                    msg: "Payment needs verification".to_string(),
                });
            }
        };
        *self.transfer.lock().await = transfer;
        *self.screen.lock().await = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use num_bigint::BigInt;
    use primitives::{Account, Asset, AssetId, Chain, FeePriority, SimulationBalanceChange, SimulationResult, SimulationWarning, TransactionInputType, Wallet, WalletId};

    use super::super::testkit::ConfirmTestkit;
    use crate::services::confirm::{GemConfirmError, GemConfirmFeeSelection, GemConfirmLoadOptions};
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
                input_type: TransactionInputType::Transfer {
                    asset: Asset::from_chain(Chain::Tron),
                },
                recipient: GemRecipient::address("THTR75o8xXAgCTQqpiot2AFRAjvW1tSbVV".into()),
                value: 0.into(),
                use_max_amount: false,
            };
            let session = testkit.service.session(wallet.clone(), transfer, None);

            let state = session.state().await.unwrap();

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
            assert!(session.load(options).await.is_err());
            assert_eq!(*testkit.balances.requests.lock().unwrap(), vec![wallet.id.clone(), wallet.id]);
        });
    }

    #[test]
    fn test_execute_refuses_a_session_without_a_preload() {
        block_on(async {
            let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Tron, "TJRyWwFs9wTFGZg3JbrVriFbNfCug5tDeC")]);
            let testkit = ConfirmTestkit::new(wallet.clone(), wallet.clone());
            let transfer = GemTransferData {
                input_type: TransactionInputType::Transfer {
                    asset: Asset::from_chain(Chain::Tron),
                },
                recipient: GemRecipient::address("THTR75o8xXAgCTQqpiot2AFRAjvW1tSbVV".into()),
                value: 0.into(),
                use_max_amount: false,
            };
            let session = testkit.service.session(wallet, transfer, None);

            assert!(matches!(session.execute().await, Err(GemConfirmError::Load { .. })));
            session.state().await.unwrap();
            assert!(matches!(session.execute().await, Err(GemConfirmError::Load { .. })));
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
                payload: vec![],
                header: None,
            };
            let transfer = GemTransferData {
                input_type: TransactionInputType::Transfer {
                    asset: Asset::from_chain(Chain::Tron),
                },
                recipient: GemRecipient::address("THTR75o8xXAgCTQqpiot2AFRAjvW1tSbVV".into()),
                value: 0.into(),
                use_max_amount: false,
            };
            let session = testkit.service.session(wallet, transfer, Some(simulation.clone()));

            let state = session.state().await.unwrap();

            assert_eq!(state.simulation.result, Some(simulation));
            assert_eq!(state.simulation.warnings.len(), 1);
            assert!(state.simulation.simulation.is_none());
            assert!(state.simulation.address_names.is_empty());
        });
    }

    #[test]
    fn test_load_switches_the_asset_only_for_a_payment() {
        block_on(async {
            let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Ethereum, "0x0000000000000000000000000000000000000001")]);
            let testkit = ConfirmTestkit::new(wallet.clone(), wallet.clone());
            let options = |asset_id: Option<primitives::AssetId>| GemConfirmLoadOptions {
                fee_selection: GemConfirmFeeSelection::Priority { priority: FeePriority::Normal },
                fee_asset_id: None,
                asset_id,
            };
            let transfer = |input_type| GemTransferData {
                input_type,
                recipient: GemRecipient::address(String::new()),
                value: 0.into(),
                use_max_amount: false,
            };
            let other = primitives::AssetId::from_chain(Chain::SmartChain);

            let sent = testkit.service.clone().session(
                wallet.clone(),
                transfer(TransactionInputType::Transfer {
                    asset: Asset::from_chain(Chain::Ethereum),
                }),
                None,
            );
            let GemConfirmError::Load { msg } = sent.load(options(Some(other.clone()))).await.unwrap_err() else {
                panic!("expected a load error");
            };
            assert_eq!(msg, "Transfer is not a payment");

            let paid = testkit.service.clone().session(
                wallet,
                transfer(TransactionInputType::Payment {
                    asset: Asset::from_chain(Chain::Ethereum),
                    invoice: primitives::PaymentInvoice::mock(),
                    extra: primitives::TransferDataExtra::mock(),
                }),
                None,
            );
            let same = paid.load(options(Some(primitives::AssetId::from_chain(Chain::Ethereum)))).await.unwrap_err();
            assert!(!same.to_string().contains("payment"), "the asset already paid with must not be re-selected: {same}");
            assert!(paid.load(options(Some(other))).await.is_err());
        });
    }
}
