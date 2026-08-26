use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::GemstoneError;
use crate::fee::custom_gas_price;
use crate::gateway::GemGateway;
use crate::models::gateway::{GemBroadcastOptions, GemFeeRate, GemTransactionPreloadInput};
use crate::models::transaction::{GemSignedTransaction, GemTransactionInputType, GemTransactionLoadFee, GemTransactionLoadInput, GemTransactionLoadMetadata};
use crate::services::GemScanService;
use crate::transaction_simulation::TransactionSimulationService;
use num_bigint::BigInt;
use primitives::{Account, ApplicationMetadataSource, AssetId, Chain, ChainType, FeePriority, ScanAddressTarget, SimulationResult, TransactionPreloadInput};
use primitives::{ScanTransaction, ScanTransactionPayload};

pub type GemAccount = Account;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct GemConfirmDestination {
    pub address: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct GemConfirmInput {
    pub input_type: GemTransactionInputType,
    pub from: GemAccount,
    pub destination: Option<GemConfirmDestination>,
    pub value: String,
    pub memo: Option<String>,
    pub references: Vec<String>,
    pub use_max: bool,
    pub minimum_value: Option<String>,
}

#[uniffi::export]
pub fn confirm_input_encode(input: &GemConfirmInput) -> Result<String, GemstoneError> {
    serde_json::to_string(input).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn confirm_input_decode(input: &str) -> Result<GemConfirmInput, GemstoneError> {
    serde_json::from_str(input).map_err(GemstoneError::from)
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemConfirmFeeSelection {
    Priority { priority: String },
    Custom { gas_price: String },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmLoadOptions {
    pub fee_selection: GemConfirmFeeSelection,
    pub fee_asset_id: Option<AssetId>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmData {
    pub fee: GemTransactionLoadFee,
    pub selected_priority: String,
    pub fee_rates: Vec<GemFeeRate>,
    pub metadata: GemTransactionLoadMetadata,
    pub scan: Option<ScanTransaction>,
    pub simulation: Option<SimulationResult>,
}

#[derive(Debug, uniffi::Error)]
pub enum GemConfirmError {
    ScanMalicious,
    ScanMemoRequired { symbol: String },
    FeeRatesMissing,
    Load { msg: String },
    Broadcast { hashes: Vec<String>, msg: String },
}

impl std::fmt::Display for GemConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ScanMalicious => write!(f, "transaction flagged as malicious"),
            Self::ScanMemoRequired { symbol } => write!(f, "{symbol} transfer requires a memo"),
            Self::FeeRatesMissing => write!(f, "fee rates not found"),
            Self::Load { msg } | Self::Broadcast { msg, .. } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemConfirmError {}

#[derive(uniffi::Object)]
pub struct GemConfirmService {
    gateway: Arc<GemGateway>,
    simulation: Arc<TransactionSimulationService>,
    scanner: Arc<GemScanService>,
}

#[uniffi::export]
impl GemConfirmService {
    #[uniffi::constructor]
    pub fn new(gateway: Arc<GemGateway>, simulation: Arc<TransactionSimulationService>, scanner: Arc<GemScanService>) -> Self {
        Self { gateway, simulation, scanner }
    }

    pub async fn load(&self, input: GemConfirmInput, options: GemConfirmLoadOptions) -> Result<GemConfirmData, GemConfirmError> {
        let asset = input.input_type.asset();
        let chain = asset.id.chain;
        let symbol = asset.symbol.clone();
        let destination = input.destination.as_ref().map(|destination| destination.address.clone()).unwrap_or_default();
        let preload_input = GemTransactionPreloadInput {
            input_type: input.input_type.clone(),
            sender_address: input.from.address.clone(),
            destination_address: destination.clone(),
            references: input.references.clone(),
        };

        let scan_future = async {
            match scan_payload(preload_input.clone()) {
                Some(payload) => self.scanner.scan_transaction(payload).await.ok(),
                None => None,
            }
        };
        let (metadata, fee_rates, scan, simulation) = futures::join!(
            self.gateway.get_transaction_preload(chain, preload_input.clone()),
            self.gateway.get_fee_rates(chain, input.input_type.clone()),
            scan_future,
            self.simulate(chain, &input),
        );
        let metadata = metadata.map_err(|error| GemConfirmError::Load { msg: error.to_string() })?;
        let fee_rates = fee_rates.map_err(|error| GemConfirmError::Load { msg: error.to_string() })?;
        let simulation = simulation?;

        validate_scan(scan.as_ref(), input.memo.as_deref(), &symbol)?;

        let selected = select_fee_rate(&fee_rates, &options.fee_selection)?;
        let load = self
            .gateway
            .get_transaction_load(
                chain,
                GemTransactionLoadInput {
                    input_type: input.input_type.clone(),
                    sender_address: input.from.address.clone(),
                    destination_address: destination,
                    value: input.value.clone(),
                    gas_price: selected.gas_price_type.clone(),
                    memo: input.memo.clone(),
                    is_max_value: input.use_max,
                    metadata,
                },
            )
            .await
            .map_err(|error| GemConfirmError::Load { msg: error.to_string() })?;

        let mut fee = load.fee;
        if let Some(fee_asset_id) = options.fee_asset_id {
            fee.fee_asset = fee_asset_id;
        }

        Ok(GemConfirmData {
            fee,
            selected_priority: selected.priority,
            fee_rates,
            metadata: load.metadata,
            scan,
            simulation,
        })
    }

    pub async fn broadcast(&self, input_type: GemTransactionInputType, transactions: Vec<GemSignedTransaction>) -> Result<Vec<String>, GemConfirmError> {
        let chain = input_type.asset().id.chain;
        let options = broadcast_options(chain, &input_type);
        let delay = broadcast_delay_milliseconds(chain);
        let mut hashes: Vec<String> = Vec::with_capacity(transactions.len());

        for (index, transaction) in transactions.iter().enumerate() {
            match self.gateway.transaction_broadcast(chain, transaction.data.clone(), options.clone()).await {
                Ok(hash) => hashes.push(hash),
                Err(error) => {
                    return Err(GemConfirmError::Broadcast { hashes, msg: error.to_string() });
                }
            }
            if index < transactions.len() - 1 && delay > 0 {
                sleep(Duration::from_millis(delay)).await;
            }
        }

        Ok(hashes)
    }
}

impl GemConfirmService {
    async fn simulate(&self, chain: Chain, input: &GemConfirmInput) -> Result<Option<SimulationResult>, GemConfirmError> {
        let (metadata, extra) = match &input.input_type {
            GemTransactionInputType::Generic { metadata, extra, .. } => (metadata, extra),
            _ => return Ok(None),
        };
        match metadata.source {
            ApplicationMetadataSource::Payment => {}
            ApplicationMetadataSource::WalletConnect => return Ok(None),
        }
        let Some(transaction) = extra.data.as_ref().and_then(|data| String::from_utf8(data.clone()).ok()) else {
            return Ok(None);
        };
        self.simulation
            .simulate_transaction(chain, transaction, Some(input.from.address.clone()))
            .await
            .map(Some)
            .map_err(|error| GemConfirmError::Load { msg: error.to_string() })
    }
}

fn broadcast_options(chain: Chain, input_type: &GemTransactionInputType) -> GemBroadcastOptions {
    match (chain, input_type) {
        (Chain::Solana, GemTransactionInputType::Swap { .. } | GemTransactionInputType::Generic { .. }) => GemBroadcastOptions { skip_preflight: true },
        _ => GemBroadcastOptions { skip_preflight: false },
    }
}

fn broadcast_delay_milliseconds(chain: Chain) -> u64 {
    match chain.chain_type() {
        ChainType::Ethereum | ChainType::HyperCore => 0,
        ChainType::Solana
        | ChainType::Bitcoin
        | ChainType::Cosmos
        | ChainType::Ton
        | ChainType::Tron
        | ChainType::Aptos
        | ChainType::Sui
        | ChainType::Near
        | ChainType::Stellar
        | ChainType::Algorand
        | ChainType::Xrp
        | ChainType::Polkadot
        | ChainType::Cardano => 500,
    }
}

async fn sleep(duration: Duration) {
    let (sender, receiver) = futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        std::thread::sleep(duration);
        let _ = sender.send(());
    });
    let _ = receiver.await;
}

fn validate_scan(scan: Option<&ScanTransaction>, memo: Option<&str>, symbol: &str) -> Result<(), GemConfirmError> {
    let Some(scan) = scan else {
        return Ok(());
    };
    if scan.is_malicious {
        return Err(GemConfirmError::ScanMalicious);
    }
    if scan.is_memo_required && memo.unwrap_or_default().trim().is_empty() {
        return Err(GemConfirmError::ScanMemoRequired { symbol: symbol.to_string() });
    }
    Ok(())
}

fn scan_payload(input: GemTransactionPreloadInput) -> Option<ScanTransactionPayload> {
    let input: TransactionPreloadInput = input.into();
    let scan_type = input.scan_type()?;
    Some(ScanTransactionPayload {
        origin: ScanAddressTarget {
            asset_id: input.input_type.get_asset().id.clone(),
            address: input.sender_address.clone(),
        },
        target: ScanAddressTarget {
            asset_id: input.input_type.get_recipient_asset().id.clone(),
            address: input.destination_address.clone(),
        },
        website: input.get_website(),
        transaction_type: scan_type,
    })
}

fn select_fee_rate(rates: &[GemFeeRate], selection: &GemConfirmFeeSelection) -> Result<GemFeeRate, GemConfirmError> {
    match selection {
        GemConfirmFeeSelection::Priority { priority } => rates
            .iter()
            .find(|rate| &rate.priority == priority)
            .or_else(|| rates.first())
            .cloned()
            .ok_or(GemConfirmError::FeeRatesMissing),
        GemConfirmFeeSelection::Custom { gas_price } => {
            let base = rates
                .iter()
                .find(|rate| rate.priority == FeePriority::Normal.as_ref())
                .or_else(|| rates.first())
                .ok_or(GemConfirmError::FeeRatesMissing)?;
            let gas_price = gas_price.parse::<BigInt>().map_err(|error| GemConfirmError::Load { msg: error.to_string() })?;
            Ok(GemFeeRate {
                priority: base.priority.clone(),
                gas_price_type: custom_gas_price(base.gas_price_type.clone(), gas_price),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::gateway::GemGasPriceType;
    use primitives::{ApplicationMetadata, Asset, TransferDataExtra, swap::SwapData};

    fn rate(priority: &str, gas_price: &str) -> GemFeeRate {
        GemFeeRate {
            priority: priority.to_string(),
            gas_price_type: GemGasPriceType::Regular { gas_price: gas_price.to_string() },
        }
    }

    #[test]
    fn test_select_fee_rate() {
        let rates = vec![rate("normal", "10"), rate("fast", "20")];

        let fast = select_fee_rate(&rates, &GemConfirmFeeSelection::Priority { priority: "fast".to_string() }).unwrap();
        assert_eq!(fast.priority, "fast");

        let fallback = select_fee_rate(&rates, &GemConfirmFeeSelection::Priority { priority: "slow".to_string() }).unwrap();
        assert_eq!(fallback.priority, "normal");

        let custom = select_fee_rate(&rates, &GemConfirmFeeSelection::Custom { gas_price: "33".to_string() }).unwrap();
        assert_eq!(custom.priority, "normal");
        match custom.gas_price_type {
            GemGasPriceType::Regular { gas_price } => assert_eq!(gas_price, "33"),
            gas_price_type => panic!("expected a regular custom gas price, got {gas_price_type:?}"),
        }

        match select_fee_rate(&[], &GemConfirmFeeSelection::Priority { priority: "normal".to_string() }) {
            Err(GemConfirmError::FeeRatesMissing) => {}
            result => panic!("expected missing fee rates, got {result:?}"),
        }
    }

    #[test]
    fn test_broadcast_policy() {
        let transfer = GemTransactionInputType::Transfer { asset: Asset::mock_sol() };
        let swap = GemTransactionInputType::Swap {
            from_asset: Asset::mock_sol(),
            to_asset: Asset::mock_spl_token(),
            swap_data: SwapData::mock(),
        };
        let payment = GemTransactionInputType::Generic {
            asset: Asset::mock_sol(),
            metadata: ApplicationMetadata::mock(),
            extra: TransferDataExtra::mock().into(),
        };

        assert!(broadcast_options(Chain::Solana, &swap).skip_preflight);
        assert!(broadcast_options(Chain::Solana, &payment).skip_preflight);
        assert!(!broadcast_options(Chain::Solana, &transfer).skip_preflight);
        assert!(!broadcast_options(Chain::Ethereum, &payment).skip_preflight);

        assert_eq!(broadcast_delay_milliseconds(Chain::Ethereum), 0);
        assert_eq!(broadcast_delay_milliseconds(Chain::HyperCore), 0);
        assert_eq!(broadcast_delay_milliseconds(Chain::Solana), 500);
    }

    #[test]
    fn test_validate_scan() {
        let safe = ScanTransaction {
            is_malicious: false,
            is_memo_required: false,
        };
        let malicious = ScanTransaction {
            is_malicious: true,
            is_memo_required: false,
        };
        let memo_required = ScanTransaction {
            is_malicious: false,
            is_memo_required: true,
        };

        assert!(validate_scan(None, None, "USDT").is_ok());
        assert!(validate_scan(Some(&safe), None, "USDT").is_ok());
        assert!(validate_scan(Some(&memo_required), Some("deposit"), "USDT").is_ok());

        match validate_scan(Some(&malicious), Some("memo"), "USDT") {
            Err(GemConfirmError::ScanMalicious) => {}
            result => panic!("expected a malicious verdict, got {result:?}"),
        }
        match validate_scan(Some(&memo_required), Some("  "), "USDT") {
            Err(GemConfirmError::ScanMemoRequired { symbol }) => assert_eq!(symbol, "USDT"),
            result => panic!("expected a required memo, got {result:?}"),
        }
    }
}
