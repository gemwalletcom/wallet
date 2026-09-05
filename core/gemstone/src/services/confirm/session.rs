use std::sync::Arc;

use futures::lock::Mutex;
use primitives::{SimulationResult, Wallet};

use super::rules::confirm_simulation;
use super::{GemConfirmError, GemConfirmFeeLoad, GemConfirmLoad, GemConfirmLoadOptions, GemConfirmTransferService};
use crate::services::transfer::GemTransferData;

#[derive(uniffi::Object)]
pub struct GemConfirmSession {
    service: Arc<GemConfirmTransferService>,
    wallet: Wallet,
    transfer: GemTransferData,
    simulation: Option<SimulationResult>,
    fee: Mutex<Option<GemConfirmFeeLoad>>,
}

impl GemConfirmSession {
    pub(super) fn new(service: Arc<GemConfirmTransferService>, wallet: Wallet, transfer: GemTransferData, simulation: Option<SimulationResult>) -> Self {
        Self {
            service,
            wallet,
            transfer,
            simulation,
            fee: Mutex::new(None),
        }
    }
}

#[uniffi::export]
impl GemConfirmSession {
    pub async fn state(&self) -> Result<GemConfirmLoad, GemConfirmError> {
        let fee = self.fee.lock().await.clone();
        self.screen(fee).await
    }

    pub async fn load(&self, options: GemConfirmLoadOptions) -> Result<GemConfirmLoad, GemConfirmError> {
        let input = self.service.confirm_input(self.wallet.clone(), self.transfer.clone())?;
        let fee = self.service.preload(self.service.wallet_id()?, input, options).await?;
        *self.fee.lock().await = Some(fee.clone());
        self.screen(Some(fee)).await
    }
}

impl GemConfirmSession {
    async fn screen(&self, fee: Option<GemConfirmFeeLoad>) -> Result<GemConfirmLoad, GemConfirmError> {
        let simulation = confirm_simulation(self.simulation.clone(), fee.as_ref().map(|fee| &fee.preload));
        self.service.state(self.transfer.clone(), simulation, fee).await
    }
}
