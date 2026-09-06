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
        let screen = self.service.state(&self.transfer, self.simulation.clone()).await?;
        *self.screen.lock().await = Some(screen.clone());
        Ok(screen)
    }

    pub async fn load(&self, options: GemConfirmLoadOptions) -> Result<GemConfirmLoad, GemConfirmError> {
        let input = self.service.confirm_input(self.wallet.clone(), self.transfer.clone())?;
        let fee = self.service.preload(self.service.wallet_id()?, input, options).await?;
        let simulation = match preload_simulation(self.simulation.as_ref(), &fee.preload) {
            Some(simulation) => Some(self.service.simulation_state(self.transfer.input_type.clone(), Some(simulation)).await?),
            None => None,
        };
        let screen = self.state().await?.with_fee(fee, simulation);
        *self.screen.lock().await = Some(screen.clone());
        Ok(screen)
    }
}
