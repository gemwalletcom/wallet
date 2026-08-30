mod error;
mod model;
pub(crate) mod rules;
mod signer;

use std::sync::Arc;
use std::time::Duration;

pub use error::GemConfirmError;
pub use model::*;
pub use rules::acquire_asset_flow;
pub use signer::GemTransactionSigner;

use crate::gateway::GemGateway;
use crate::models::gateway::GemTransactionPreloadInput;
use crate::models::transaction::{GemSignedTransaction, GemTransactionInputType, GemTransactionLoadInput};
use crate::services::GemScanService;
use crate::services::clock::sleep;
use crate::services::transaction_state::GemTransactionStateService;
use crate::signer::GemSignerError;
use crate::transaction_simulation::TransactionSimulationService;
use primitives::{Chain, SimulationResult, Transaction, TransferDataOutputAction};

#[derive(uniffi::Object)]
pub struct GemConfirmService {
    gateway: Arc<GemGateway>,
    simulation: Arc<TransactionSimulationService>,
    scanner: Arc<GemScanService>,
    transaction_state: Arc<GemTransactionStateService>,
}

#[uniffi::export]
impl GemConfirmService {
    #[uniffi::constructor]
    pub fn new(gateway: Arc<GemGateway>, simulation: Arc<TransactionSimulationService>, scanner: Arc<GemScanService>, transaction_state: Arc<GemTransactionStateService>) -> Self {
        Self {
            gateway,
            simulation,
            scanner,
            transaction_state,
        }
    }

    pub async fn execute(&self, input: GemSendInput, signer: Arc<dyn GemTransactionSigner>) -> Result<GemExecuteResult, GemConfirmError> {
        let transactions = signer.sign(input.wallet.clone(), rules::signer_input(&input)?).await?;
        if transactions.is_empty() {
            return Err(GemConfirmError::Sign {
                error: GemSignerError::SigningError("no signed transactions".to_string()),
                msg: "no signed transactions".to_string(),
            });
        }
        rules::validate_approvals(&input.confirm.input.transfer.input_type, &transactions)?;
        match rules::output_action(&input.confirm.input.transfer.input_type) {
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
        let fee_rates = fee_rates.map_err(error::load_error)?;
        let simulation = simulation?;

        rules::validate_scan(scan.as_ref(), transfer.recipient.memo.as_deref(), &symbol)?;

        let selected = rules::select_fee_rate(&fee_rates, &options.fee_selection)?;
        let load = self
            .gateway
            .get_transaction_load(
                chain,
                GemTransactionLoadInput {
                    input_type: transfer.input_type.clone(),
                    sender_address: input.from.address.clone(),
                    destination_address: destination,
                    value: transfer.value.to_string(),
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
    async fn send(&self, input: GemSendInput, transactions: Vec<GemSignedTransaction>) -> Result<GemSendResult, GemConfirmError> {
        let hashes = match self.broadcast(input.confirm.input.transfer.input_type.clone(), transactions.clone()).await {
            Ok(hashes) => hashes,
            Err(GemConfirmError::Broadcast { hashes, msg }) => {
                self.record(&input, &hashes, &transactions).await?;
                return Err(GemConfirmError::Broadcast { hashes, msg });
            }
            Err(error) => return Err(error),
        };
        let transactions = self.record(&input, &hashes, &transactions).await?;
        Ok(GemSendResult { hashes, transactions })
    }

    async fn broadcast(&self, input_type: GemTransactionInputType, transactions: Vec<GemSignedTransaction>) -> Result<Vec<String>, GemConfirmError> {
        let chain = input_type.asset().id.chain;
        let options = rules::broadcast_options(chain, &input_type);
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

    async fn record(&self, input: &GemSendInput, hashes: &[String], transactions: &[GemSignedTransaction]) -> Result<Vec<Transaction>, GemConfirmError> {
        let pending = rules::pending_transactions(input, hashes, transactions)?;
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
        let Some(transaction) = rules::simulation_payload(&input.transfer.input_type) else {
            return Ok(None);
        };
        self.simulation
            .simulate_transaction(chain, transaction, Some(input.from.address.clone()))
            .await
            .map(Some)
            .map_err(|error| GemConfirmError::Load { msg: error.to_string() })
    }
}
