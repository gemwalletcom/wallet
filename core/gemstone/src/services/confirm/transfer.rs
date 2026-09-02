use std::sync::{Arc, Mutex};

use primitives::currency::Currency;
use primitives::{AddressName, Chain, PerpetualModifyConfirmData, SimulationResult, Transaction, WalletId};

use crate::block_explorer::GemBlockExplorerLink;
use crate::models::transaction::GemTransactionInputType;
use crate::services::assets::config::GemAssetConfigService;
use crate::services::confirm::rules::is_insufficient_network_fee;
use crate::services::confirm::{
    GemAcquireAssetFlow, GemConfirmError, GemConfirmInput, GemConfirmLoadOptions, GemConfirmMetadata, GemConfirmPreload, GemConfirmSceneLoad, GemConfirmSceneState,
    GemConfirmService, GemConfirmSimulationState, GemExecuteResult, GemFeeAsset, GemSendInput, GemTransactionSigner,
};
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::name::GemNameService;
use crate::services::perpetual::model::GemAutocloseSummary;
use crate::services::perpetual::rules::autoclose_summary;
use crate::services::preferences::GemPreferencesService;
use crate::services::transfer::GemRecentActivityService;
use crate::services::wallet::{GemKeystoreAuthentication, GemKeystorePassword};

#[derive(uniffi::Object)]
pub struct GemConfirmTransferService {
    confirm: Arc<GemConfirmService>,
    explorer: Arc<GemExplorerService>,
    names: Arc<GemNameService>,
    asset_config: Arc<GemAssetConfigService>,
    signer: Arc<dyn GemTransactionSigner>,
    password: Arc<dyn GemKeystorePassword>,
    recent_activity: Arc<GemRecentActivityService>,
    preferences: Arc<GemPreferencesService>,
    scene: Mutex<Option<GemConfirmSceneLoad>>,
}

#[uniffi::export]
impl GemConfirmTransferService {
    #[uniffi::constructor]
    pub fn new(
        confirm: Arc<GemConfirmService>,
        explorer: Arc<GemExplorerService>,
        names: Arc<GemNameService>,
        asset_config: Arc<GemAssetConfigService>,
        signer: Arc<dyn GemTransactionSigner>,
        password: Arc<dyn GemKeystorePassword>,
        recent_activity: Arc<GemRecentActivityService>,
        preferences: Arc<GemPreferencesService>,
    ) -> Self {
        Self {
            confirm,
            explorer,
            names,
            asset_config,
            signer,
            password,
            recent_activity,
            preferences,
            scene: Mutex::new(None),
        }
    }

    pub fn currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn authentication(&self) -> GemKeystoreAuthentication {
        self.password.authentication().unwrap_or(GemKeystoreAuthentication::None)
    }

    pub fn metadata(&self, wallet_id: WalletId, input_type: GemTransactionInputType) -> Result<GemConfirmMetadata, GemConfirmError> {
        self.confirm.input_metadata(wallet_id, &input_type, input_type.fee_asset().id)
    }

    pub fn scene_state(&self, wallet_id: WalletId, input_type: GemTransactionInputType, simulation: Option<SimulationResult>) -> GemConfirmSceneState {
        GemConfirmSceneState {
            fee_priority: input_type.default_fee_priority(),
            fee_asset: input_type.fee_asset(),
            metadata: self.metadata(wallet_id, input_type.clone()).ok(),
            simulation: self.confirm.simulation(input_type, simulation).ok(),
        }
    }

    pub async fn execute(&self, input: GemSendInput) -> Result<GemExecuteResult, GemConfirmError> {
        let wallet_id = input.wallet.id.clone();
        let input_type = input.confirm.input.transfer.input_type.clone();
        let result = self.confirm.execute(input, self.signer.clone()).await?;
        if is_broadcast(&result) {
            let _ = self.recent_activity.add(input_type, wallet_id).await;
        }
        Ok(result)
    }

    pub async fn load(
        &self,
        wallet_id: WalletId,
        input: GemConfirmInput,
        options: GemConfirmLoadOptions,
        simulation: Option<SimulationResult>,
    ) -> Result<GemConfirmSceneLoad, GemConfirmError> {
        let loaded = self.scene.lock().expect("confirm scene lock").clone();
        let scene = match loaded {
            Some(previous) => GemConfirmSceneLoad {
                preload: self.preload(wallet_id, input, options).await?,
                fee_assets: previous.fee_assets,
                simulation: previous.simulation,
            },
            None => self.load_scene(wallet_id, input, options, simulation).await?,
        };
        *self.scene.lock().expect("confirm scene lock") = Some(scene.clone());
        Ok(scene)
    }

    pub async fn track_pending(&self) -> Result<(), GemServiceError> {
        self.confirm.track_pending().await
    }

    pub async fn track(&self, wallet_id: WalletId, transactions: Vec<Transaction>) -> Result<(), GemServiceError> {
        self.confirm.track(wallet_id, transactions).await
    }

    pub fn address_url(&self, chain: Chain, address: String) -> GemBlockExplorerLink {
        self.explorer.get_address_url(chain, address)
    }

    pub fn address_name(&self, chain: Chain, address: String) -> Result<Option<AddressName>, GemServiceError> {
        self.names.address_name(chain, address)
    }

    pub fn autoclose_summary(&self, data: PerpetualModifyConfirmData) -> Option<GemAutocloseSummary> {
        autoclose_summary(&data)
    }

    pub fn acquire_asset_flow(&self, chain: Chain) -> GemAcquireAssetFlow {
        self.asset_config.acquire_flow(chain)
    }
}

fn is_broadcast(result: &GemExecuteResult) -> bool {
    match result {
        GemExecuteResult::Sent { .. } => true,
        GemExecuteResult::Signed { .. } => false,
    }
}

impl GemConfirmTransferService {
    fn fee_assets(&self, wallet_id: WalletId, chain: Chain) -> Result<Vec<GemFeeAsset>, GemConfirmError> {
        self.confirm.fee_assets(wallet_id, chain)
    }

    async fn preload(&self, wallet_id: WalletId, input: GemConfirmInput, options: GemConfirmLoadOptions) -> Result<GemConfirmPreload, GemConfirmError> {
        let input_type = input.transfer.input_type.clone();
        match self.confirm.preload(wallet_id.clone(), input, options).await {
            Ok(preload) => Ok(preload),
            Err(error) => Err(self.missing_network_fee(wallet_id, input_type).unwrap_or(error)),
        }
    }

    async fn load_scene(
        &self,
        wallet_id: WalletId,
        input: GemConfirmInput,
        options: GemConfirmLoadOptions,
        simulation: Option<SimulationResult>,
    ) -> Result<GemConfirmSceneLoad, GemConfirmError> {
        let input_type = input.transfer.input_type.clone();
        let fee_assets = self.fee_assets(wallet_id.clone(), input_type.transaction_asset().chain())?;
        let preload = self.preload(wallet_id, input, options).await?;
        let simulation = simulation.or_else(|| preload.confirm_data.simulation.clone());
        Ok(GemConfirmSceneLoad {
            fee_assets,
            simulation: self.simulation_state(input_type, simulation).await,
            preload,
        })
    }

    async fn simulation_state(&self, input_type: GemTransactionInputType, simulation: Option<SimulationResult>) -> GemConfirmSimulationState {
        if let Some(simulation) = &simulation {
            let _ = self.confirm.sync_missing_assets(simulation.asset_ids()).await;
        }
        let Ok(details) = self.confirm.simulation(input_type.clone(), simulation) else {
            return GemConfirmSimulationState {
                simulation: None,
                address_names: Vec::new(),
            };
        };
        let requests = details.address_requests(input_type.transaction_asset().chain());
        let address_names = self.names.get_address_names(requests).await.unwrap_or_default();
        GemConfirmSimulationState {
            simulation: Some(details),
            address_names,
        }
    }

    fn missing_network_fee(&self, wallet_id: WalletId, input_type: GemTransactionInputType) -> Option<GemConfirmError> {
        let balance = self.metadata(wallet_id, input_type).ok()?.fee_asset_balance;
        is_insufficient_network_fee(balance.asset_id.clone(), &balance.available.to_string()).then_some(GemConfirmError::InsufficientNetworkFee { asset_id: balance.asset_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_only_a_broadcast_send_records_recent_activity() {
        let sent = GemExecuteResult::Sent {
            hashes: vec!["0xhash".to_string()],
            transactions: vec![],
        };
        let signed = GemExecuteResult::Signed {
            data: vec!["0xsigned".to_string()],
        };

        assert!(is_broadcast(&sent));
        assert!(!is_broadcast(&signed));
    }
}
