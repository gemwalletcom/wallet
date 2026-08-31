use std::sync::Arc;

use primitives::{AddressName, Asset, AssetId, Chain, ChainAddress, PerpetualModifyConfirmData, SimulationResult, Transaction, TransactionType, WalletId};

use crate::application::GemApplicationMetadataService;
use crate::block_explorer::GemBlockExplorerLink;
use crate::fee::GemFeeService;
use crate::models::transaction::GemTransactionInputType;
use crate::services::assets::config::GemAssetConfigService;
use crate::services::confirm::{
    GemAcquireAssetFlow, GemConfirmError, GemConfirmInput, GemConfirmLoadOptions, GemConfirmMetadata, GemConfirmPreload, GemConfirmService, GemConfirmSimulation, GemExecuteResult,
    GemFeeAsset, GemSendInput, GemTransactionSigner,
};
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::name::GemNameService;
use crate::services::perpetual::model::GemAutocloseSummary;
use crate::services::perpetual::rules::autoclose_summary;
use crate::services::swap::config::GemSwapQuoteService;
use crate::services::transfer::GemTransferService;

#[derive(uniffi::Object)]
pub struct GemConfirmSceneService {
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
impl GemConfirmSceneService {
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

    pub fn metadata(&self, wallet_id: WalletId, asset_id: AssetId, fee_asset_id: AssetId, extra_asset_ids: Vec<AssetId>) -> Result<GemConfirmMetadata, GemConfirmError> {
        self.confirm.metadata(wallet_id, asset_id, fee_asset_id, extra_asset_ids)
    }

    pub fn fee_assets(&self, wallet_id: WalletId, chain: Chain) -> Result<Vec<GemFeeAsset>, GemConfirmError> {
        self.confirm.fee_assets(wallet_id, chain)
    }

    pub fn simulation(&self, input_type: GemTransactionInputType, simulation: Option<SimulationResult>) -> Result<GemConfirmSimulation, GemConfirmError> {
        self.confirm.simulation(input_type, simulation)
    }

    pub async fn preload(&self, wallet_id: WalletId, input: GemConfirmInput, options: GemConfirmLoadOptions) -> Result<GemConfirmPreload, GemConfirmError> {
        self.confirm.preload(wallet_id, input, options).await
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

    pub fn fee_asset(&self, input_type: GemTransactionInputType) -> Asset {
        self.transfer.fee_asset(&input_type)
    }

    pub fn asset_ids(&self, input_type: GemTransactionInputType) -> Vec<AssetId> {
        self.transfer.asset_ids(&input_type)
    }

    pub fn transaction_type(&self, input_type: GemTransactionInputType) -> TransactionType {
        self.transfer.transaction_type(&input_type)
    }

    pub fn autoclose_summary(&self, data: PerpetualModifyConfirmData) -> Option<GemAutocloseSummary> {
        autoclose_summary(&data)
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

    pub fn application_metadata(&self) -> Arc<GemApplicationMetadataService> {
        self.application_metadata.clone()
    }

    pub fn transfer(&self) -> Arc<GemTransferService> {
        self.transfer.clone()
    }
}
