use std::sync::{Arc, Mutex};

use primitives::currency::Currency;
use primitives::{Asset, Chain, PerpetualModifyConfirmData, SimulationResult, Wallet, WalletId};

use crate::block_explorer::GemBlockExplorerLink;
use crate::models::custom_types::GemBigInt;
use crate::models::transaction::GemTransactionInputType;
use crate::services::assets::config::GemAssetConfigService;
use crate::services::confirm::rules::is_insufficient_network_fee;
use crate::services::confirm::{
    GemAcquireAssetFlow, GemConfirmData, GemConfirmError, GemConfirmInitialState, GemConfirmInput, GemConfirmLoad, GemConfirmLoadOptions, GemConfirmMetadata, GemConfirmPreload,
    GemConfirmService, GemConfirmSimulationState, GemExecuteResult, GemFeeAsset, GemTransactionSigner, SendInput,
};
use crate::services::explorer::GemExplorerService;
use crate::services::name::GemNameService;
use crate::services::perpetual::model::GemAutocloseSummary;
use crate::services::perpetual::rules::autoclose_summary;
use crate::services::preferences::GemPreferencesService;
use crate::services::transfer::{GemRecentActivityService, GemTransferData};
use crate::services::wallet::{GemKeystoreAuthentication, GemKeystorePassword};
use crate::services::wallet_session::GemWalletSessionService;

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
    session: Arc<GemWalletSessionService>,
    loaded: Mutex<Option<GemConfirmLoad>>,
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
        session: Arc<GemWalletSessionService>,
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
            session,
            loaded: Mutex::new(None),
        }
    }

    pub fn get_currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn authentication(&self) -> GemKeystoreAuthentication {
        self.password.authentication().unwrap_or(GemKeystoreAuthentication::None)
    }

    pub fn confirm_input(&self, wallet: Wallet, transfer: GemTransferData) -> Result<GemConfirmInput, GemConfirmError> {
        let chain = transfer.input_type.asset().chain();
        let from = wallet.account(chain).cloned().ok_or(GemConfirmError::AccountMissing { chain })?;
        Ok(GemConfirmInput { from, transfer })
    }

    pub fn initial_state(&self, input_type: GemTransactionInputType, simulation: Option<SimulationResult>) -> GemConfirmInitialState {
        GemConfirmInitialState {
            fee_priority: input_type.default_fee_priority(),
            fee_asset: input_type.fee_asset(),
            simulation: self.confirm.simulation(input_type, simulation, Vec::new()).ok(),
        }
    }

    pub async fn execute(
        &self,
        confirm: GemConfirmData,
        value: GemBigInt,
        network_fee: GemBigInt,
        simulation: Option<SimulationResult>,
    ) -> Result<GemExecuteResult, GemConfirmError> {
        let wallet = self.wallet().await?;
        let wallet_id = wallet.id.clone();
        let input_type = confirm.input.transfer.input_type.clone();
        let input = SendInput {
            wallet,
            confirm,
            value,
            network_fee,
            simulation,
        };
        let result = self.confirm.execute(input, self.signer.clone()).await?;
        if is_broadcast(&result) {
            let _ = self.recent_activity.add(input_type, wallet_id).await;
        }
        Ok(result)
    }

    pub async fn load(&self, input: GemConfirmInput, options: GemConfirmLoadOptions, simulation: Option<SimulationResult>) -> Result<GemConfirmLoad, GemConfirmError> {
        let wallet_id = self.wallet_id()?;
        let previous = self.loaded.lock().expect("confirm load lock").clone();
        let load = match previous {
            Some(previous) => GemConfirmLoad {
                preload: self.preload(wallet_id, input, options).await?,
                fee_assets: previous.fee_assets,
                simulation: previous.simulation,
                address_name: previous.address_name,
            },
            None => self.initial_load(wallet_id, input, options, simulation).await?,
        };
        *self.loaded.lock().expect("confirm load lock") = Some(load.clone());
        Ok(load)
    }

    pub fn address_url(&self, chain: Chain, address: String) -> GemBlockExplorerLink {
        self.explorer.get_address_url(chain, address)
    }

    pub fn autoclose_summary(&self, data: PerpetualModifyConfirmData) -> Option<GemAutocloseSummary> {
        autoclose_summary(&data)
    }

    pub fn acquire_asset_flow(&self, chain: Chain) -> GemAcquireAssetFlow {
        self.asset_config.acquire_flow(chain)
    }
}

impl GemConfirmTransferService {
    pub async fn metadata(&self, input_type: GemTransactionInputType) -> Result<GemConfirmMetadata, GemConfirmError> {
        self.confirm.input_metadata(self.wallet_id()?, &input_type, input_type.fee_asset().id).await
    }
}

fn is_broadcast(result: &GemExecuteResult) -> bool {
    match result {
        GemExecuteResult::Sent { .. } => true,
        GemExecuteResult::Signed { .. } => false,
    }
}

impl GemConfirmTransferService {
    async fn wallet(&self) -> Result<Wallet, GemConfirmError> {
        Ok(self.session.current_wallet().await?)
    }

    fn wallet_id(&self) -> Result<WalletId, GemConfirmError> {
        Ok(self.session.current_wallet_id()?)
    }

    async fn fee_assets(&self, wallet_id: WalletId, chain: Chain) -> Result<Vec<GemFeeAsset>, GemConfirmError> {
        self.confirm.fee_assets(wallet_id, chain).await
    }

    async fn preload(&self, wallet_id: WalletId, input: GemConfirmInput, options: GemConfirmLoadOptions) -> Result<GemConfirmPreload, GemConfirmError> {
        let input_type = input.transfer.input_type.clone();
        match self.confirm.preload(wallet_id.clone(), input, options).await {
            Ok(preload) => Ok(preload),
            Err(error) => Err(self.missing_network_fee(wallet_id, input_type).await.unwrap_or(error)),
        }
    }

    async fn initial_load(
        &self,
        wallet_id: WalletId,
        input: GemConfirmInput,
        options: GemConfirmLoadOptions,
        simulation: Option<SimulationResult>,
    ) -> Result<GemConfirmLoad, GemConfirmError> {
        let input_type = input.transfer.input_type.clone();
        let recipient = input.transfer.recipient.address.clone();
        let chain = input_type.transaction_asset().chain();
        let fee_assets = self.fee_assets(wallet_id.clone(), chain).await?;
        let address_name = self.names.address_name(chain, recipient).await.unwrap_or_default();
        let preload = self.preload(wallet_id, input, options).await?;
        let simulation = simulation.or_else(|| preload.confirm_data.simulation.clone());
        Ok(GemConfirmLoad {
            fee_assets,
            simulation: self.simulation_state(input_type, simulation).await?,
            preload,
            address_name,
        })
    }

    async fn simulation_state(&self, input_type: GemTransactionInputType, simulation: Option<SimulationResult>) -> Result<GemConfirmSimulationState, GemConfirmError> {
        let assets = match &simulation {
            Some(simulation) => self.confirm.ensure_simulation_assets(simulation.asset_ids()).await?,
            None => Vec::new(),
        };
        let Ok(details) = self.confirm.simulation(input_type.clone(), simulation, assets) else {
            return Ok(GemConfirmSimulationState {
                simulation: None,
                address_names: Vec::new(),
            });
        };
        let requests = details.address_requests(input_type.transaction_asset().chain());
        let address_names = self.names.get_address_names(requests).await.unwrap_or_default();
        Ok(GemConfirmSimulationState {
            simulation: Some(details),
            address_names,
        })
    }

    async fn missing_network_fee(&self, wallet_id: WalletId, input_type: GemTransactionInputType) -> Option<GemConfirmError> {
        let balance = self.confirm.input_metadata(wallet_id, &input_type, input_type.fee_asset().id).await.ok()?.fee_asset_balance;
        is_insufficient_network_fee(balance.asset_id.clone(), &balance.available.to_string()).then(|| GemConfirmError::InsufficientNetworkFee {
            asset: Asset::from_chain(balance.asset_id.chain),
            requirement: None,
        })
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
