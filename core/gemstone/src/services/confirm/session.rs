use std::sync::Arc;

use futures::lock::Mutex;
use primitives::{SimulationResult, Wallet};

use super::rules::preload_simulation;
use super::{GemConfirmError, GemConfirmLoad, GemConfirmLoadOptions, GemConfirmTransferService};
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
        let fee = self.service.preload(self.wallet.id.clone(), input, options).await?;
        let simulation = match preload_simulation(self.simulation.as_ref(), &fee.preload) {
            Some(simulation) => Some(self.service.simulation_state(self.transfer.input_type.clone(), Some(simulation)).await?),
            None => None,
        };
        let screen = self.state().await?.with_fee(fee, simulation);
        *self.screen.lock().await = Some(screen.clone());
        Ok(screen)
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use primitives::{Account, Asset, Chain, FeePriority, TransactionInputType, Wallet, WalletId};

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
}
