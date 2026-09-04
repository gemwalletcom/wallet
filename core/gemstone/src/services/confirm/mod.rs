#![allow(clippy::result_large_err)]

mod error;
mod model;
pub(crate) mod rules;
mod signer;
mod transfer;

use std::sync::Arc;
use std::time::Duration;

pub use error::GemConfirmError;
pub use model::*;
pub use rules::acquire_asset_flow;
pub use signer::GemTransactionSigner;
pub use transfer::GemConfirmTransferService;

use crate::gateway::GemGateway;
use crate::models::asset::chain_fee_asset_ids;
use crate::models::gateway::GemTransactionPreloadInput;
use crate::models::transaction::{GemSignedTransaction, GemTransactionInputType, GemTransactionLoadInput};
use crate::services::GemScanService;
use crate::services::assets::GemAssetsService;
use crate::services::balance::GemBalanceService;
use crate::services::clock::sleep;
use crate::services::price::GemPriceService;
use crate::services::simulation::{GemSimulationFormatter, GemSimulationService};
use crate::services::transaction_state::{GemTransactionStateService, GemTransactionStatusService};
use crate::signer::GemSignerError;
use primitives::{Asset, AssetId, Chain, SimulationPayloadFieldDisplay, SimulationResult, Transaction, TransferDataOutputAction, WalletId};

#[derive(uniffi::Object)]
pub struct GemConfirmService {
    gateway: Arc<GemGateway>,
    simulation: Arc<GemSimulationService>,
    scanner: Arc<GemScanService>,
    transaction_state: Arc<GemTransactionStateService>,
    balance: Arc<GemBalanceService>,
    price: Arc<GemPriceService>,
    assets: Arc<GemAssetsService>,
    transaction_status: Arc<dyn GemTransactionStatusService>,
    simulation_formatter: GemSimulationFormatter,
}

#[uniffi::export]
impl GemConfirmService {
    #[uniffi::constructor]
    pub fn new(
        gateway: Arc<GemGateway>,
        simulation: Arc<GemSimulationService>,
        scanner: Arc<GemScanService>,
        transaction_state: Arc<GemTransactionStateService>,
        balance: Arc<GemBalanceService>,
        price: Arc<GemPriceService>,
        assets: Arc<GemAssetsService>,
        transaction_status: Arc<dyn GemTransactionStatusService>,
    ) -> Self {
        Self {
            gateway,
            simulation,
            scanner,
            transaction_state,
            balance,
            price,
            assets,
            transaction_status,
            simulation_formatter: GemSimulationFormatter::new(),
        }
    }

    pub async fn metadata(&self, wallet_id: WalletId, asset_id: AssetId, fee_asset_id: AssetId, extra_asset_ids: Vec<AssetId>) -> Result<GemConfirmMetadata, GemConfirmError> {
        let asset_ids = rules::metadata_asset_ids(&asset_id, &fee_asset_id, extra_asset_ids);
        let balances = self.balance.balances(wallet_id, asset_ids.clone()).await?;
        let prices = self.price.prices(asset_ids).await?;
        rules::build_metadata(asset_id, fee_asset_id, balances, prices)
    }

    pub async fn sync_missing_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, crate::services::error::GemServiceError> {
        self.assets.sync_missing_assets(asset_ids).await
    }

    pub async fn load(&self, input: GemConfirmInput, options: GemConfirmLoadOptions) -> Result<GemConfirmData, GemConfirmError> {
        let transfer = &input.transfer;
        let asset = transfer.input_type.asset();
        let chain = asset.id.chain;
        let symbol = asset.symbol.clone();
        let destination = transfer.recipient.address.clone();
        let preload_input = GemTransactionPreloadInput {
            input_type: transfer.input_type.clone(),
            sender_address: input.from.address.clone(),
            destination_address: destination.clone(),
            references: transfer.recipient.references.clone(),
        };

        // A scanner outage fails open by design: the send continues without a verdict.
        let scan_future = async { self.scanner.scan_transaction(rules::scan_payload(preload_input.clone())).await.ok() };
        let (metadata, fee_rates, scan, simulation) = futures::join!(
            self.gateway.get_transaction_preload(chain, preload_input.clone()),
            self.gateway.get_fee_rates(chain, transfer.input_type.clone()),
            scan_future,
            self.simulate(chain, &input),
        );
        let metadata = metadata.map_err(error::load_error)?;
        let fee_rates = rules::displayed_fee_rates(fee_rates.map_err(error::load_error)?);
        let simulation = simulation?;

        rules::validate_scan(scan.as_ref(), transfer.recipient.memo.as_deref(), &symbol)?;

        let selected = options.fee_selection.select_fee_rate(&fee_rates)?;
        let load = self
            .gateway
            .get_transaction_load(
                chain,
                GemTransactionLoadInput {
                    input_type: transfer.input_type.clone(),
                    sender_address: input.from.address.clone(),
                    destination_address: destination,
                    value: transfer.value.to_biguint().ok_or_else(|| GemConfirmError::Load {
                        msg: "negative transfer value".to_string(),
                    })?,
                    gas_price: selected.gas_price_type.clone(),
                    memo: transfer.recipient.memo.clone(),
                    is_max_value: transfer.use_max_amount,
                    metadata,
                },
            )
            .await
            .map_err(error::load_error)?;

        let mut fee = load.fee;
        if let Some(fee_asset_id) = options.fee_asset_id {
            fee.fee_asset = fee_asset_id;
        }

        Ok(GemConfirmData {
            input,
            fee,
            selected_priority: selected.priority,
            fee_rates,
            metadata: load.metadata,
            simulation,
        })
    }
}

impl GemConfirmService {
    pub fn simulation(&self, input_type: GemTransactionInputType, simulation: Option<SimulationResult>, assets: Vec<Asset>) -> Result<GemConfirmSimulation, GemConfirmError> {
        let has_critical_warning = simulation.as_ref().map(SimulationResult::has_critical_warning).unwrap_or(false);
        let approval = input_type.approval_value();
        let shows_header = self.simulation_formatter.shows_header(simulation.clone(), approval.is_some());
        let payload_fields = self
            .simulation_formatter
            .payload_fields(simulation.clone().map(|simulation| simulation.payload).unwrap_or_default(), shows_header);
        let header = match approval {
            Some((asset_id, value)) => assets
                .iter()
                .find(|asset| asset.id == asset_id)
                .map(|asset| GemSimulationValue { asset: asset.clone(), value }),
            None => self.simulation_formatter.header(simulation.clone()).and_then(|header| {
                assets.iter().find(|asset| asset.id == header.asset_id).map(|asset| GemSimulationValue {
                    asset: asset.clone(),
                    value: rules::approval_value_from(&header.value, header.is_unlimited),
                })
            }),
        };
        let balance_changes = self
            .simulation_formatter
            .balance_changes(simulation, assets.iter().map(|asset| asset.id.clone()).collect())
            .into_iter()
            .filter_map(|change| {
                let asset = assets.iter().find(|asset| asset.id == change.asset_id)?.clone();
                Some(GemSimulationBalanceChange { asset, value: change.value })
            })
            .collect();
        Ok(GemConfirmSimulation {
            has_critical_warning,
            primary_fields: payload_fields
                .iter()
                .filter(|field| field.display == SimulationPayloadFieldDisplay::Primary)
                .cloned()
                .collect(),
            secondary_fields: payload_fields
                .iter()
                .filter(|field| field.display == SimulationPayloadFieldDisplay::Secondary)
                .cloned()
                .collect(),
            header,
            balance_changes,
        })
    }
}

impl GemConfirmService {
    pub async fn execute(&self, input: SendInput, signer: Arc<dyn GemTransactionSigner>) -> Result<GemExecuteResult, GemConfirmError> {
        let signer_input = input.signer_input()?;
        let chain = input.confirm.input.transfer.input_type.asset().chain();
        let transactions = signer.sign(input.wallet.clone(), signer_input).await.map_err(|error| error::sign_error(chain, error))?;
        if transactions.is_empty() {
            return Err(GemConfirmError::Sign {
                error: GemSignerError::SigningError("no signed transactions".to_string()),
                chain,
                msg: "no signed transactions".to_string(),
            });
        }
        input.confirm.input.transfer.input_type.validate_approvals(&transactions)?;
        match input.confirm.input.transfer.input_type.output().output_action {
            TransferDataOutputAction::Sign => Ok(GemExecuteResult::Signed {
                data: transactions.into_iter().map(|transaction| transaction.data).collect(),
            }),
            TransferDataOutputAction::Send => {
                let result = self.send(input, transactions).await?;
                Ok(GemExecuteResult::Sent {
                    hashes: result.hashes,
                    transactions: result.transactions,
                })
            }
        }
    }
}

impl GemConfirmService {
    async fn ensure_simulation_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, crate::services::error::GemServiceError> {
        self.assets.ensure_simulation_assets(asset_ids).await
    }

    pub async fn fee_assets(&self, wallet_id: WalletId, chain: Chain) -> Result<Vec<GemFeeAsset>, GemConfirmError> {
        let fee_asset_ids = chain_fee_asset_ids(chain);
        if fee_asset_ids.is_empty() {
            return Ok(Vec::new());
        }
        let assets = self.assets.assets(fee_asset_ids.clone()).await?;
        let balances = self.balance.balances(wallet_id, fee_asset_ids.clone()).await?;
        let prices = self.price.prices(fee_asset_ids).await?;
        Ok(rules::selectable_fee_assets(assets, balances, prices))
    }
    pub async fn preload(&self, wallet_id: WalletId, input: GemConfirmInput, options: GemConfirmLoadOptions) -> Result<GemConfirmPreload, GemConfirmError> {
        let confirm_data = self.load(input, options).await?;
        let fee_asset_id = confirm_data.fee.fee_asset.clone();
        let metadata = self
            .input_metadata(wallet_id.clone(), &confirm_data.input.transfer.input_type, fee_asset_id.clone())
            .await?;
        let fee_asset = self
            .assets
            .assets(vec![fee_asset_id.clone()])
            .await?
            .into_iter()
            .next()
            .ok_or(GemConfirmError::BalanceMissing { asset_id: fee_asset_id.clone() })?;
        let amount = confirm_data.preload_amount(&metadata, &fee_asset)?;
        Ok(GemConfirmPreload {
            confirm_data,
            metadata,
            fee_asset,
            amount,
        })
    }
}

impl GemConfirmService {
    async fn send(&self, input: SendInput, signed: Vec<GemSignedTransaction>) -> Result<GemSendResult, GemConfirmError> {
        match self.broadcast(input.confirm.input.transfer.input_type.clone(), signed.clone()).await {
            Ok(hashes) => {
                let transactions = self.store_pending(&input, &hashes, &signed).await;
                Ok(GemSendResult { hashes, transactions })
            }
            Err(GemConfirmError::Broadcast { hashes, msg }) => {
                self.store_pending(&input, &hashes, &signed).await;
                Err(GemConfirmError::Broadcast { hashes, msg })
            }
            Err(error) => Err(error),
        }
    }

    async fn store_pending(&self, input: &SendInput, hashes: &[String], signed: &[GemSignedTransaction]) -> Vec<Transaction> {
        let stored = self.record(input, hashes, signed).await.unwrap_or_default();
        self.transaction_status.track(input.wallet.id.clone(), stored.clone());
        stored
    }

    async fn broadcast(&self, input_type: GemTransactionInputType, transactions: Vec<GemSignedTransaction>) -> Result<Vec<String>, GemConfirmError> {
        let chain = input_type.asset().id.chain;
        let options = input_type.broadcast_options();
        let delay = rules::broadcast_delay_milliseconds(chain);
        let mut hashes: Vec<String> = Vec::with_capacity(transactions.len());

        for (index, transaction) in transactions.iter().enumerate() {
            match self.gateway.transaction_broadcast(chain, transaction.data.clone(), options.clone()).await {
                Ok(hash) => hashes.push(hash),
                Err(error) => {
                    return Err(error::broadcast_error(hashes, error));
                }
            }
            if index < transactions.len() - 1 && delay > 0 {
                sleep(Duration::from_millis(delay)).await;
            }
        }

        Ok(hashes)
    }

    async fn record(&self, input: &SendInput, hashes: &[String], signed: &[GemSignedTransaction]) -> Result<Vec<Transaction>, GemConfirmError> {
        let pending = input.pending_transactions(hashes, signed)?;
        if pending.is_empty() {
            return Ok(pending);
        }
        self.transaction_state
            .add_transactions(input.wallet.id.clone(), pending.clone())
            .await
            .map_err(|error| GemConfirmError::Record { msg: error.to_string() })?;
        Ok(pending)
    }

    async fn simulate(&self, chain: Chain, input: &GemConfirmInput) -> Result<Option<SimulationResult>, GemConfirmError> {
        let Some(transaction) = input.transfer.input_type.simulation_payload() else {
            return Ok(None);
        };
        self.simulation
            .simulate_transaction(chain, transaction, Some(input.from.address.clone()))
            .await
            .map(Some)
            .map_err(|error| GemConfirmError::Load { msg: error.to_string() })
    }
}

impl GemConfirmService {
    async fn input_metadata(&self, wallet_id: WalletId, input_type: &GemTransactionInputType, fee_asset_id: AssetId) -> Result<GemConfirmMetadata, GemConfirmError> {
        self.metadata(wallet_id, input_type.transaction_asset().id, fee_asset_id, input_type.asset_ids()).await
    }
}
