use std::sync::Arc;

use futures::lock::Mutex;
use primitives::{SimulationResult, Wallet};

use super::rules::preload_simulation;
use super::{GemConfirmError, GemConfirmLoad, GemConfirmLoadOptions, GemConfirmScreen, GemConfirmTransferService};
use crate::services::transfer::GemTransferData;

#[derive(uniffi::Object)]
pub struct GemConfirmSession {
    service: Arc<GemConfirmTransferService>,
    wallet: Wallet,
    transfer: GemTransferData,
    simulation: Option<SimulationResult>,
    screen: Mutex<Option<GemConfirmLoad>>,
}

impl GemConfirmSession {
    pub(super) fn new(service: Arc<GemConfirmTransferService>, wallet: Wallet, transfer: GemTransferData, simulation: Option<SimulationResult>) -> Self {
        Self {
            service,
            wallet,
            transfer,
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

    pub async fn state(&self) -> Result<GemConfirmLoad, GemConfirmError> {
        if let Some(screen) = self.screen.lock().await.clone() {
            return Ok(screen);
        }
        let input = self.service.confirm_input(self.wallet.clone(), self.transfer.clone())?;
        let screen = self.service.state(self.wallet.id.clone(), &input, self.simulation.clone()).await?;
        *self.screen.lock().await = Some(screen.clone());
        Ok(screen)
    }

    pub async fn load(&self, options: GemConfirmLoadOptions) -> Result<GemConfirmLoad, GemConfirmError> {
        let input = self.service.confirm_input(self.wallet.clone(), self.transfer.clone())?;
        let input_type = self.transfer.input_type.clone();
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

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use num_bigint::BigInt;
    use primitives::{Account, Asset, AssetId, Chain, FeePriority, SimulationBalanceChange, SimulationResult, SimulationWarning, TransactionInputType, Wallet, WalletId};

    use super::super::testkit::ConfirmTestkit;
    use crate::services::confirm::{GemConfirmFeeSelection, GemConfirmLoadOptions};
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
            };
            assert!(session.load(options).await.is_err());
            assert_eq!(*testkit.balances.requests.lock().unwrap(), vec![wallet.id.clone(), wallet.id]);
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
}
