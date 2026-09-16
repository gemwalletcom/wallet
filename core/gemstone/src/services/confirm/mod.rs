#![allow(clippy::result_large_err)]

mod confirmation;
mod error;
mod model;
pub(crate) mod rules;
pub(crate) mod screen;
mod signer;
#[cfg(test)]
mod testkit;
mod transfer;

use std::sync::Arc;
use std::time::Duration;

pub use confirmation::GemConfirmation;
pub use error::GemConfirmError;
pub use model::*;
pub use rules::acquire_asset_flow;
pub use signer::GemTransactionSigner;
pub use transfer::GemConfirmTransferService;

use crate::gateway::GemGateway;
use crate::models::asset::chain_fee_asset_ids;
use crate::models::gateway::GemTransactionPreloadInput;
use crate::models::transaction::{GemSignedTransaction, GemTransactionData, GemTransactionLoadInput};
use crate::services::GemScanService;
use crate::services::assets::GemAssetsService;
use crate::services::balance::GemBalanceService;
use crate::services::clock::sleep;
use crate::services::confirm::rules::ConfirmInput;
use crate::services::price::GemPriceService;
use crate::services::simulation::{GemSimulationFormatter, GemSimulationService};
use crate::services::transaction_state::{GemTransactionStateService, GemTransactionStatusService};
use crate::services::transfer::rules::TransferInput;
use crate::signer::GemSignerError;
use primitives::TransactionInputType;
use num_bigint::BigInt;
use primitives::{Asset, AssetId, Chain, SimulationPayloadFieldDisplay, SimulationResult, Transaction, TransactionFee, WalletId};

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
    pub async fn metadata(&self, wallet_id: WalletId, asset_id: AssetId, fee_asset_id: AssetId, extra_asset_ids: Vec<AssetId>) -> Result<GemConfirmMetadata, GemConfirmError> {
        let asset_ids = rules::metadata_asset_ids(&asset_id, &fee_asset_id, extra_asset_ids);
        let (balances, prices) = futures::join!(self.balance.balances(wallet_id, asset_ids.clone()), self.price.prices(asset_ids));
        rules::build_metadata(asset_id, fee_asset_id, balances?, prices?)
    }

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

    pub async fn sync_missing_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, crate::services::error::GemServiceError> {
        self.assets.sync_missing_assets(asset_ids).await
    }

    pub async fn load(&self, input: GemConfirmInput, options: GemConfirmLoadOptions) -> Result<GemConfirmData, GemConfirmError> {
        let transfer = &input.transfer;
        let asset = transfer.input_type.get_asset();
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
        let scan_future = async {
            let payload = rules::scan_payload(preload_input.clone())?;
            self.scanner.scan_transaction(payload).await.ok()
        };
        let (metadata, fee_rates, scan, simulation) = futures::join!(
            self.gateway.get_transaction_preload(chain, preload_input.clone()),
            self.gateway.get_fee_rates(chain, transfer.input_type.clone()),
            scan_future,
            self.simulate(chain, &input),
        );
        let metadata = metadata.map_err(error::load_error)?;
        let fee_rates = rules::confirmation_fee_rates(chain, transfer.use_max_amount, fee_rates.map_err(error::load_error)?);
        let simulation = simulation?;

        rules::validate_scan(scan.as_ref(), transfer.recipient.memo.as_deref(), &symbol)?;

        let selected = options.fee_selection.select_fee_rate(&fee_rates)?;
        let load = if rules::is_signature_only(&transfer.input_type) {
            GemTransactionData {
                fee: TransactionFee::new_from_fee(BigInt::ZERO, AssetId::from_chain(chain)).into(),
                metadata,
            }
        } else {
            self.gateway
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
                .map_err(error::load_error)?
        };

        let mut fee = load.fee;
        if let Some(fee_asset_id) = options.fee_asset_id.filter(|fee_asset_id| fee_asset_id.chain == chain) {
            fee.fee_asset = fee_asset_id;
        }

        Ok(GemConfirmData {
            additional_fees: fee.options.items(),
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
    pub fn simulation(&self, input_type: TransactionInputType, simulation: Option<SimulationResult>, assets: Vec<Asset>) -> Result<GemConfirmSimulation, GemConfirmError> {
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
            None => simulation.as_ref().and_then(|simulation| GemSimulationValue::from_simulation(simulation, &assets)),
        };
        let balance_changes = self
            .simulation_formatter
            .balance_changes(simulation, assets.iter().map(|asset| asset.id.clone()).collect())
            .into_iter()
            .filter_map(|change| {
                let asset = assets.iter().find(|asset| asset.id == change.asset_id)?.clone();
                Some(GemSimulationBalanceChange {
                    asset,
                    sign: rules::balance_change_sign(&change.value),
                    value: change.value,
                })
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
    async fn sign(&self, input: &SendInput, signer: Arc<dyn GemTransactionSigner>) -> Result<Vec<GemSignedTransaction>, GemConfirmError> {
        let signer_input = input.signer_input()?;
        let chain = input.confirm.input.transfer.input_type.get_asset().chain();
        let transactions = signer.sign(input.wallet.clone(), signer_input).await.map_err(|error| error::sign_error(chain, error))?;
        if transactions.is_empty() {
            return Err(GemConfirmError::Sign {
                error: GemSignerError::SigningError("no signed transactions".to_string()),
                chain,
                msg: "no signed transactions".to_string(),
            });
        }
        input.confirm.input.transfer.input_type.validate_approvals(&transactions)?;
        Ok(transactions)
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
        let (assets, balances, prices) = futures::join!(
            self.assets.assets(fee_asset_ids.clone()),
            self.balance.balances(wallet_id, fee_asset_ids.clone()),
            self.price.prices(fee_asset_ids),
        );
        Ok(rules::selectable_fee_assets(assets?, balances?, prices?))
    }
    pub async fn preload(&self, wallet_id: WalletId, input: GemConfirmInput, options: GemConfirmLoadOptions) -> Result<GemConfirmFeeLoad, GemConfirmError> {
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
        Ok(GemConfirmFeeLoad {
            fee_asset,
            metadata,
            preload: GemConfirmPreload { confirm_data, amount },
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

    async fn broadcast(&self, input_type: TransactionInputType, transactions: Vec<GemSignedTransaction>) -> Result<Vec<String>, GemConfirmError> {
        let chain = input_type.get_asset().id.chain;
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
        let simulation = self
            .simulation
            .simulate_transaction(chain, transaction, Some(input.from.address.clone()))
            .await
            .map_err(|error| GemConfirmError::Load { msg: error.to_string() })?;
        input.transfer.input_type.validate_simulation(&simulation)?;
        Ok(Some(simulation))
    }
}

impl GemConfirmService {
    async fn input_metadata(&self, wallet_id: WalletId, input_type: &TransactionInputType, fee_asset_id: AssetId) -> Result<GemConfirmMetadata, GemConfirmError> {
        self.metadata(wallet_id, input_type.transaction_asset().id, fee_asset_id, input_type.asset_ids()).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use futures::executor::block_on;
    use primitives::{Account, Asset, Chain, FeePriority, TransactionInputType, Wallet, asset_constants::HYPERCORE_SPOT_USDC_ASSET_ID, swap::SwapData};

    use super::testkit::ConfirmTestkit;
    use super::{GemConfirmData, GemConfirmError, GemConfirmFeeSelection, GemConfirmLoadOptions};
    use crate::services::transfer::{GemRecipient, GemTransferData};
    use crate::testkit::TestAlienProvider;

    fn load_with_scan(scan: &str) -> (Result<GemConfirmData, GemConfirmError>, Vec<String>) {
        block_on(async {
            let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::HyperCore, "0xsender")]);
            let provider = Arc::new(TestAlienProvider::with_json(200, scan));
            let testkit = ConfirmTestkit::with_provider(wallet.clone(), wallet.clone(), provider.clone());
            let transfer = GemTransferData {
                recipient: GemRecipient::address("0xrecipient".into()),
                value: 0.into(),
                ..GemTransferData::mock(TransactionInputType::Swap {
                    from_asset: Asset::from_chain(Chain::HyperCore),
                    to_asset: Asset {
                        id: HYPERCORE_SPOT_USDC_ASSET_ID.clone(),
                        ..Asset::from_chain(Chain::HyperCore)
                    },
                    swap_data: SwapData::mock(),
                })
            };
            let input = testkit.service.confirm_input(wallet, transfer).unwrap();
            let options = GemConfirmLoadOptions {
                fee_selection: GemConfirmFeeSelection::Priority { priority: FeePriority::Normal },
                fee_asset_id: None,
                asset_id: None,
            };

            let result = testkit.confirm.load(input, options).await;
            (result, provider.requested_paths())
        })
    }

    #[test]
    fn test_a_malicious_verdict_stops_the_load_before_it_asks_the_chain() {
        let (result, requests) = load_with_scan(r#"{"isMalicious":true,"isScanComplete":true}"#);

        assert!(matches!(result, Err(GemConfirmError::ScanMalicious)));
        assert_eq!(
            requests,
            vec!["/v2/devices/scan/transaction"],
            "a rejected input never reaches the transaction load, so the chain is never asked and no agent credential is created for it"
        );
    }

    #[test]
    fn test_a_clean_verdict_lets_the_load_ask_the_chain() {
        let (_, requests) = load_with_scan(r#"{"isMalicious":false,"isScanComplete":true}"#);

        assert_eq!(
            requests,
            vec!["/v2/devices/scan/transaction", "https://gemnodes.com/hypercore/info"],
            "the transaction load runs once the scan clears, and only then"
        );
    }
}
