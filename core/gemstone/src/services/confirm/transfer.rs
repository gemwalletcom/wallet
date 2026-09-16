use crate::services::simulation::warning_rows;
use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{Asset, Chain, PerpetualModifyConfirmData, SimulationResult, Wallet, WalletId};

use crate::config::fiat_config::get_fiat_config;
use crate::models::custom_types::GemBigInt;
use crate::models::transaction::GemSignedTransaction;
use crate::payment::GemPaymentService;
use crate::services::assets::config::GemAssetConfigService;
use crate::services::confirm::rules::{is_broadcast, is_insufficient_network_fee};
use crate::services::confirm::{
    GemAcquireAssetFlow, GemConfirmData, GemConfirmError, GemConfirmFeeLoad, GemConfirmInput, GemConfirmLoad, GemConfirmLoadOptions, GemConfirmService, GemConfirmSimulationState,
    GemConfirmation, GemExecuteResult, GemFeeAsset, GemTransactionSigner, SendInput,
};
use crate::services::explorer::GemExplorerService;
use crate::services::name::GemNameService;
use crate::services::perpetual::model::GemAutocloseSummary;
use crate::services::perpetual::rules::autoclose_summary;
use crate::services::preferences::GemPreferencesService;
use crate::services::transfer::rules::TransferInput;
use crate::services::transfer::{GemRecentActivityService, GemTransferData};
use crate::services::wallet::{GemKeystoreAuthentication, GemKeystorePassword};
use primitives::BlockExplorerLink;
use primitives::TransactionInputType;

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
    payment: Arc<GemPaymentService>,
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
        payment: Arc<GemPaymentService>,
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
            payment,
        }
    }

    pub fn confirmation(self: Arc<Self>, wallet: Wallet, transfer: GemTransferData, simulation: Option<SimulationResult>) -> Arc<GemConfirmation> {
        Arc::new(GemConfirmation::new(self, wallet, transfer, simulation))
    }

    pub fn address_url(&self, chain: Chain, address: String) -> BlockExplorerLink {
        self.explorer.get_address_url(chain, address)
    }
}

fn simulation_seed(chain: Chain, simulation: Option<SimulationResult>) -> GemConfirmSimulationState {
    GemConfirmSimulationState {
        chain,
        warnings: simulation.as_ref().map(|result| warning_rows(&result.warnings)).unwrap_or_default(),
        result: simulation,
        simulation: None,
        address_names: Vec::new(),
    }
}

impl GemConfirmTransferService {
    pub(super) fn get_currency(&self) -> Currency {
        self.preferences.get_currency()
    }
    pub(super) fn authentication(&self) -> GemKeystoreAuthentication {
        self.password.authentication().unwrap_or(GemKeystoreAuthentication::None)
    }
    pub(super) fn autoclose_summary(&self, data: PerpetualModifyConfirmData) -> Option<GemAutocloseSummary> {
        autoclose_summary(&data)
    }
    pub(super) fn acquire_asset_flow(&self, chain: Chain) -> GemAcquireAssetFlow {
        self.asset_config.acquire_flow(chain)
    }
    pub(super) fn insufficient_network_fee_buy_amount(&self) -> i32 {
        get_fiat_config().insufficient_network_fee_buy_amount
    }

    pub(super) async fn execute(
        &self,
        wallet: Wallet,
        confirm: GemConfirmData,
        value: GemBigInt,
        network_fee: GemBigInt,
        simulation: Option<SimulationResult>,
    ) -> Result<GemExecuteResult, GemConfirmError> {
        let wallet_id = wallet.id.clone();
        let input_type = confirm.input.transfer.input_type.clone();
        let input = SendInput {
            wallet,
            confirm,
            value,
            network_fee,
            simulation,
        };
        let signed = self.confirm.sign(&input, self.signer.clone()).await?;
        let (transactions, signatures): (Vec<GemSignedTransaction>, Vec<GemSignedTransaction>) =
            signed.into_iter().partition(|transaction| is_broadcast(&input_type, transaction));
        let data: Vec<String> = signatures.iter().map(|transaction| transaction.data.clone()).collect();
        if transactions.is_empty() {
            let warning = self.report_payment(&input, data.clone(), &signatures).await;
            return Ok(GemExecuteResult::Signed { data, warning });
        }
        let sent = self.confirm.send(input.clone(), transactions).await?;
        let _ = self.recent_activity.add(input_type, wallet_id).await;
        let warning = self.report_payment(&input, [sent.hashes.clone(), data].concat(), &signatures).await;
        Ok(GemExecuteResult::Sent {
            hashes: sent.hashes,
            transactions: sent.transactions,
            warning,
        })
    }

    async fn report_payment(&self, input: &SendInput, action_results: Vec<String>, signatures: &[GemSignedTransaction]) -> Option<String> {
        let input_type = &input.confirm.input.transfer.input_type;
        let TransactionInputType::Payment { .. } = input_type else {
            return None;
        };
        if let Err(error) = self.payment.confirm(input_type, action_results).await {
            return Some(error.to_string());
        }
        if let Some(hash) = self.payment.record_hash(input_type)
            && !signatures.is_empty()
        {
            let record = SendInput {
                network_fee: GemBigInt::from(0),
                ..input.clone()
            };
            self.confirm.store_pending(&record, &[hash], signatures).await;
        }
        None
    }

    pub(super) fn payment(&self) -> &GemPaymentService {
        &self.payment
    }

    pub(super) fn confirm_input(&self, wallet: Wallet, transfer: GemTransferData) -> Result<GemConfirmInput, GemConfirmError> {
        let chain = transfer.input_type.get_asset().chain();
        let from = wallet.account(chain).cloned().ok_or(GemConfirmError::AccountMissing { chain })?;
        Ok(GemConfirmInput { from, transfer })
    }

    async fn fee_assets(&self, wallet_id: WalletId, chain: Chain) -> Result<Vec<GemFeeAsset>, GemConfirmError> {
        self.confirm.fee_assets(wallet_id, chain).await
    }

    pub(super) async fn preload(&self, wallet_id: WalletId, input: GemConfirmInput, options: GemConfirmLoadOptions) -> Result<GemConfirmFeeLoad, GemConfirmError> {
        let input_type = input.transfer.input_type.clone();
        match self.confirm.preload(wallet_id.clone(), input, options).await {
            Ok(fee) => Ok(fee),
            Err(error) => Err(self.missing_network_fee(wallet_id, input_type).await.unwrap_or(error)),
        }
    }

    pub(super) async fn state(&self, wallet_id: WalletId, input: &GemConfirmInput, simulation: Option<SimulationResult>) -> Result<GemConfirmLoad, GemConfirmError> {
        let input_type = input.transfer.input_type.clone();
        let chain = input_type.transaction_asset().chain();
        let (metadata, fee_assets, address_name) = futures::join!(
            self.confirm.input_metadata(wallet_id.clone(), &input_type, input_type.fee_asset().id),
            self.fee_assets(wallet_id, chain),
            self.names.address_name(chain, input.transfer.recipient.address.clone()),
        );
        Ok(GemConfirmLoad {
            transfer: input.transfer.clone(),
            sender: input.from.clone(),
            fee_asset: input_type.fee_asset(),
            metadata: metadata?,
            fee_assets: fee_assets?,
            simulation: simulation_seed(chain, simulation),
            address_name: address_name.unwrap_or_default(),
            preload: None,
        })
    }

    pub(super) async fn simulation_state(&self, input_type: TransactionInputType, simulation: Option<SimulationResult>) -> Result<GemConfirmSimulationState, GemConfirmError> {
        let chain = input_type.transaction_asset().chain();
        let assets = match &simulation {
            Some(simulation) => self.confirm.ensure_simulation_assets(simulation.asset_ids()).await?,
            None => Vec::new(),
        };
        let Ok(details) = self.confirm.simulation(input_type, simulation.clone(), assets) else {
            return Ok(simulation_seed(chain, simulation));
        };
        let requests = details.address_requests(chain);
        let address_names = self.names.get_address_names(requests).await.unwrap_or_default();
        Ok(GemConfirmSimulationState {
            chain,
            warnings: simulation.as_ref().map(|result| warning_rows(&result.warnings)).unwrap_or_default(),
            result: simulation,
            simulation: Some(details),
            address_names,
        })
    }

    async fn missing_network_fee(&self, wallet_id: WalletId, input_type: TransactionInputType) -> Option<GemConfirmError> {
        let balance = self.confirm.input_metadata(wallet_id, &input_type, input_type.fee_asset().id).await.ok()?.fee_asset_balance;
        is_insufficient_network_fee(&balance.asset_id, &balance.available).then(|| GemConfirmError::InsufficientNetworkFee {
            asset: Asset::from_chain(balance.asset_id.chain),
            requirement: None,
        })
    }
}

