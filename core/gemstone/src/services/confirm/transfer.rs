use std::sync::Arc;

use primitives::{AddressName, AssetId, Chain, ChainAddress, PerpetualModifyConfirmData, SimulationResult, Transaction, TransactionType, TransferDataOutputAction, WalletId};

use crate::application::GemApplicationMetadataService;
use crate::block_explorer::GemBlockExplorerLink;
use crate::fee::GemFeeService;
use crate::models::transaction::GemTransactionInputType;
use crate::services::assets::config::GemAssetConfigService;
use crate::services::confirm::{
    GemAcquireAssetFlow, GemConfirmError, GemConfirmInput, GemConfirmLoadOptions, GemConfirmMetadata, GemConfirmPreload, GemConfirmSceneState, GemConfirmService,
    GemConfirmSimulation, GemExecuteResult, GemFeeAsset, GemSendInput, GemTransactionSigner,
};
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::name::GemNameService;
use crate::services::perpetual::model::GemAutocloseSummary;
use crate::services::perpetual::rules::autoclose_summary;
use crate::services::swap::config::GemSwapQuoteService;
use crate::services::transfer::{GemRecentActivity, GemTransferService};

#[derive(uniffi::Object)]
pub struct GemConfirmTransferService {
    confirm: Arc<GemConfirmService>,
    explorer: Arc<GemExplorerService>,
    names: Arc<GemNameService>,
    asset_config: Arc<GemAssetConfigService>,
    transfer: Arc<GemTransferService>,
    fee: Arc<GemFeeService>,
    swap_quote: Arc<GemSwapQuoteService>,
    application_metadata: Arc<GemApplicationMetadataService>,
}

#[uniffi::export]
impl GemConfirmTransferService {
    #[uniffi::constructor]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        confirm: Arc<GemConfirmService>,
        explorer: Arc<GemExplorerService>,
        names: Arc<GemNameService>,
        asset_config: Arc<GemAssetConfigService>,
        transfer: Arc<GemTransferService>,
        fee: Arc<GemFeeService>,
        swap_quote: Arc<GemSwapQuoteService>,
        application_metadata: Arc<GemApplicationMetadataService>,
    ) -> Self {
        Self {
            confirm,
            explorer,
            names,
            asset_config,
            transfer,
            fee,
            swap_quote,
            application_metadata,
        }
    }

    pub fn metadata(&self, wallet_id: WalletId, input_type: GemTransactionInputType) -> Result<GemConfirmMetadata, GemConfirmError> {
        self.confirm.metadata(
            wallet_id,
            self.transfer.asset(&input_type).id,
            self.transfer.fee_asset(&input_type).id,
            self.transfer.asset_ids(&input_type),
        )
    }

    pub fn scene_state(&self, wallet_id: WalletId, input_type: GemTransactionInputType, simulation: Option<SimulationResult>) -> GemConfirmSceneState {
        GemConfirmSceneState {
            fee_priority: self.fee.default_priority(input_type.clone()),
            fee_asset: self.transfer.fee_asset(&input_type),
            metadata: self.metadata(wallet_id, input_type.clone()).ok(),
            simulation: self.confirm.simulation(input_type, simulation).ok(),
        }
    }

    pub fn fee_assets(&self, wallet_id: WalletId, chain: Chain) -> Result<Vec<GemFeeAsset>, GemConfirmError> {
        self.confirm.fee_assets(wallet_id, chain)
    }

    pub fn simulation(&self, input_type: GemTransactionInputType, simulation: Option<SimulationResult>) -> Result<GemConfirmSimulation, GemConfirmError> {
        self.confirm.simulation(input_type, simulation)
    }

    pub async fn preload(&self, wallet_id: WalletId, input: GemConfirmInput, options: GemConfirmLoadOptions) -> Result<GemConfirmPreload, GemConfirmError> {
        let input_type = input.transfer.input_type.clone();
        match self.confirm.preload(wallet_id.clone(), input, options).await {
            Ok(preload) => Ok(preload),
            Err(error) => Err(self.missing_network_fee(wallet_id, input_type).unwrap_or(error)),
        }
    }

    pub async fn execute(&self, input: GemSendInput, signer: Arc<dyn GemTransactionSigner>) -> Result<GemExecuteResult, GemConfirmError> {
        self.confirm.execute(input, signer).await
    }

    pub async fn sync_missing_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemServiceError> {
        self.confirm.sync_missing_assets(asset_ids).await
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

    pub async fn address_names(&self, requests: Vec<ChainAddress>) -> Result<Vec<AddressName>, GemServiceError> {
        self.names.get_address_names(requests).await
    }

    pub fn default_fee_priority(&self, input_type: GemTransactionInputType) -> primitives::FeePriority {
        self.fee.default_priority(input_type)
    }

    pub fn is_insufficient_network_fee(&self, fee_asset_id: AssetId, fee_available: String) -> bool {
        self.fee.is_insufficient_network_fee(fee_asset_id, fee_available)
    }

    pub fn transaction_type(&self, input_type: GemTransactionInputType) -> TransactionType {
        self.transfer.transaction_type(&input_type)
    }

    pub fn autoclose_summary(&self, data: PerpetualModifyConfirmData) -> Option<GemAutocloseSummary> {
        autoclose_summary(&data)
    }

    pub fn application_short_name(&self, input_type: GemTransactionInputType) -> Option<String> {
        match input_type {
            GemTransactionInputType::Generic { metadata, .. } => Some(self.application_metadata.short_name(metadata)),
            _ => None,
        }
    }

    pub fn recent_activity(&self, input_type: GemTransactionInputType) -> Option<GemRecentActivity> {
        self.transfer.recent_activity(&input_type)
    }

    pub fn output_action(&self, input_type: GemTransactionInputType) -> TransferDataOutputAction {
        self.transfer.output(&input_type).output_action
    }

    pub fn acquire_asset_flow(&self, chain: Chain) -> GemAcquireAssetFlow {
        self.asset_config.acquire_flow(chain)
    }

    pub fn fee(&self) -> Arc<GemFeeService> {
        self.fee.clone()
    }

    pub fn swap_quote(&self) -> Arc<GemSwapQuoteService> {
        self.swap_quote.clone()
    }
}

impl GemConfirmTransferService {
    fn missing_network_fee(&self, wallet_id: WalletId, input_type: GemTransactionInputType) -> Option<GemConfirmError> {
        let balance = self.metadata(wallet_id, input_type).ok()?.fee_asset_balance;
        self.fee
            .is_insufficient_network_fee(balance.asset_id.clone(), balance.available.to_string())
            .then_some(GemConfirmError::InsufficientNetworkFee { asset_id: balance.asset_id })
    }
}
