use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::GemstoneError;
use crate::fee::custom_gas_price;
use crate::gateway::{GatewayError, GemGateway};
use crate::models::gateway::{GemBroadcastOptions, GemFeeRate, GemTransactionPreloadInput};
use crate::models::transaction::{GemSignedTransaction, GemTransactionInputType, GemTransactionLoadFee, GemTransactionLoadInput, GemTransactionLoadMetadata};
use crate::services::GemScanService;
use crate::services::transfer::GemTransferData;
use crate::transaction_simulation::TransactionSimulationService;
use num_bigint::BigInt;
use primitives::{Account, ApplicationMetadataSource, AssetId, Chain, ChainType, FeePriority, ScanAddressTarget, SimulationResult, TransactionPreloadInput};
use primitives::{ScanTransaction, ScanTransactionPayload};

pub type GemAccount = Account;

#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct GemConfirmInput {
    pub from: GemAccount,
    pub transfer: GemTransferData,
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
    Offline,
    Network { msg: String },
    Load { msg: String },
    Broadcast { hashes: Vec<String>, msg: String },
}

impl std::fmt::Display for GemConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ScanMalicious => write!(f, "transaction flagged as malicious"),
            Self::ScanMemoRequired { symbol } => write!(f, "{symbol} transfer requires a memo"),
            Self::FeeRatesMissing => write!(f, "fee rates not found"),
            Self::Offline => write!(f, "network offline"),
            Self::Network { msg } | Self::Load { msg } | Self::Broadcast { msg, .. } => write!(f, "{msg}"),
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
        let scan_future = async { self.scanner.scan_transaction(scan_payload(preload_input.clone())).await.ok() };
        let (metadata, fee_rates, scan, simulation) = futures::join!(
            self.gateway.get_transaction_preload(chain, preload_input.clone()),
            self.gateway.get_fee_rates(chain, transfer.input_type.clone()),
            scan_future,
            self.simulate(chain, &input),
        );
        let metadata = metadata.map_err(load_error)?;
        let fee_rates = fee_rates.map_err(load_error)?;
        let simulation = simulation?;

        validate_scan(scan.as_ref(), transfer.recipient.memo.as_deref(), &symbol)?;

        let selected = select_fee_rate(&fee_rates, &options.fee_selection)?;
        let load = self
            .gateway
            .get_transaction_load(
                chain,
                GemTransactionLoadInput {
                    input_type: transfer.input_type.clone(),
                    sender_address: input.from.address.clone(),
                    destination_address: destination,
                    value: transfer.value.clone(),
                    gas_price: selected.gas_price_type.clone(),
                    memo: transfer.recipient.memo.clone(),
                    is_max_value: transfer.use_max_amount,
                    metadata,
                },
            )
            .await
            .map_err(load_error)?;

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
                    return Err(broadcast_error(hashes, error));
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
        let Some(transaction) = simulation_payload(&input.transfer.input_type) else {
            return Ok(None);
        };
        self.simulation
            .simulate_transaction(chain, transaction, Some(input.from.address.clone()))
            .await
            .map(Some)
            .map_err(|error| GemConfirmError::Load { msg: error.to_string() })
    }
}

#[uniffi::export]
pub fn default_fee_priority(input_type: GemTransactionInputType) -> String {
    let priority = match input_type {
        GemTransactionInputType::Swap { from_asset, .. } if from_asset.chain() == Chain::Bitcoin => FeePriority::Fast,
        _ => FeePriority::Normal,
    };
    priority.as_ref().to_string()
}

#[uniffi::export]
pub fn is_insufficient_network_fee(fee_asset_id: AssetId, fee_available: String) -> bool {
    if matches!(fee_asset_id.chain, Chain::HyperCore | Chain::Tron) || !fee_asset_id.is_native() {
        return false;
    }
    fee_available.trim().is_empty() || fee_available.trim().chars().all(|character| character == '0')
}

fn simulation_payload(input_type: &GemTransactionInputType) -> Option<String> {
    let GemTransactionInputType::Generic { metadata, extra, .. } = input_type else {
        return None;
    };
    match metadata.source {
        ApplicationMetadataSource::Payment => extra.data.as_ref().and_then(|data| String::from_utf8(data.clone()).ok()),
        ApplicationMetadataSource::WalletConnect => None,
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

fn load_error(error: GatewayError) -> GemConfirmError {
    match error {
        GatewayError::Offline => GemConfirmError::Offline,
        GatewayError::NetworkError { msg } => GemConfirmError::Network { msg },
        error => GemConfirmError::Load { msg: error.to_string() },
    }
}

fn broadcast_error(hashes: Vec<String>, error: GatewayError) -> GemConfirmError {
    match error {
        GatewayError::Offline if hashes.is_empty() => GemConfirmError::Offline,
        GatewayError::NetworkError { msg } if hashes.is_empty() => GemConfirmError::Network { msg },
        error => GemConfirmError::Broadcast { hashes, msg: error.to_string() },
    }
}

fn scan_payload(input: GemTransactionPreloadInput) -> ScanTransactionPayload {
    let input: TransactionPreloadInput = input.into();
    ScanTransactionPayload {
        origin: ScanAddressTarget {
            asset_id: input.input_type.get_asset().id.clone(),
            address: input.sender_address.clone(),
        },
        target: ScanAddressTarget {
            asset_id: input.input_type.get_recipient_asset().id.clone(),
            address: input.destination_address.clone(),
        },
        website: input.get_website(),
        transaction_type: input.input_type.transaction_type(),
    }
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
    use primitives::{
        ApplicationMetadata, Asset, PerpetualConfirmData, PerpetualDirection, PerpetualType, StakeType, TransactionType, TransferDataExtra,
        swap::{ApprovalData, SwapData},
    };

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
    fn test_select_fee_rate_custom() {
        let eip1559 = GemFeeRate {
            priority: "normal".to_string(),
            gas_price_type: GemGasPriceType::Eip1559 {
                gas_price: "20".to_string(),
                priority_fee: "5".to_string(),
            },
        };
        let rates = vec![rate("slow", "1"), eip1559];

        let raised = select_fee_rate(&rates, &GemConfirmFeeSelection::Custom { gas_price: "30".to_string() }).unwrap();
        assert_eq!(raised.priority, "normal");
        match raised.gas_price_type {
            GemGasPriceType::Eip1559 { gas_price, priority_fee } => assert_eq!((gas_price.as_str(), priority_fee.as_str()), ("25", "5")),
            gas_price_type => panic!("expected an eip1559 custom gas price, got {gas_price_type:?}"),
        }

        let capped = select_fee_rate(&rates, &GemConfirmFeeSelection::Custom { gas_price: "3".to_string() }).unwrap();
        match capped.gas_price_type {
            GemGasPriceType::Eip1559 { gas_price, priority_fee } => assert_eq!((gas_price.as_str(), priority_fee.as_str()), ("0", "3")),
            gas_price_type => panic!("expected a capped eip1559 gas price, got {gas_price_type:?}"),
        }

        let without_normal = select_fee_rate(&[rate("slow", "1"), rate("fast", "9")], &GemConfirmFeeSelection::Custom { gas_price: "4".to_string() }).unwrap();
        assert_eq!(without_normal.priority, "slow");
        match without_normal.gas_price_type {
            GemGasPriceType::Regular { gas_price } => assert_eq!(gas_price, "4"),
            gas_price_type => panic!("expected a regular custom gas price, got {gas_price_type:?}"),
        }

        match select_fee_rate(&rates, &GemConfirmFeeSelection::Custom { gas_price: "abc".to_string() }) {
            Err(GemConfirmError::Load { .. }) => {}
            result => panic!("expected a load error for a malformed gas price, got {result:?}"),
        }
        match select_fee_rate(&[], &GemConfirmFeeSelection::Custom { gas_price: "1".to_string() }) {
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

        let approve = GemTransactionInputType::TokenApprove {
            asset: Asset::mock_sol(),
            approval_data: ApprovalData::mock(),
        };
        let stake = GemTransactionInputType::Stake {
            asset: Asset::mock_sol(),
            stake_type: StakeType::Rewards(vec![]),
        };
        let perpetual = GemTransactionInputType::Perpetual {
            asset: Asset::mock_sol(),
            perpetual_type: PerpetualType::Open(PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None)),
        };
        let ethereum_swap = GemTransactionInputType::Swap {
            from_asset: Asset::mock(),
            to_asset: Asset::mock_erc20(),
            swap_data: SwapData::mock(),
        };

        assert!(broadcast_options(Chain::Solana, &swap).skip_preflight);
        assert!(broadcast_options(Chain::Solana, &payment).skip_preflight);
        assert!(!broadcast_options(Chain::Solana, &transfer).skip_preflight);
        assert!(!broadcast_options(Chain::Solana, &approve).skip_preflight);
        assert!(!broadcast_options(Chain::Solana, &stake).skip_preflight);
        assert!(!broadcast_options(Chain::Solana, &perpetual).skip_preflight);
        assert!(!broadcast_options(Chain::Ethereum, &payment).skip_preflight);
        assert!(!broadcast_options(Chain::Ethereum, &ethereum_swap).skip_preflight);

        assert_eq!(broadcast_delay_milliseconds(Chain::Ethereum), 0);
        assert_eq!(broadcast_delay_milliseconds(Chain::HyperCore), 0);
        assert_eq!(broadcast_delay_milliseconds(Chain::Solana), 500);
        for chain in Chain::all() {
            let is_instant = matches!(chain.chain_type(), ChainType::Ethereum | ChainType::HyperCore);
            assert_eq!(broadcast_delay_milliseconds(chain), if is_instant { 0 } else { 500 }, "{chain}");
        }
    }

    #[test]
    fn test_scan_payload_covers_every_input_type() {
        let swap = GemTransactionPreloadInput {
            input_type: GemTransactionInputType::Swap {
                from_asset: Asset::mock_sol(),
                to_asset: Asset::mock_spl_token(),
                swap_data: SwapData::mock(),
            },
            sender_address: "sender".to_string(),
            destination_address: "router".to_string(),
            references: vec![],
        };
        let payload = scan_payload(swap);
        assert_eq!(payload.transaction_type, TransactionType::Swap);
        assert_eq!(payload.origin.asset_id, Asset::mock_sol().id);
        assert_eq!(payload.target.asset_id, Asset::mock_spl_token().id);
        assert_eq!(payload.target.address, "router");
        assert_eq!(payload.website, None);

        let generic = GemTransactionPreloadInput {
            input_type: GemTransactionInputType::Generic {
                asset: Asset::mock_sol(),
                metadata: ApplicationMetadata::mock(),
                extra: TransferDataExtra::mock().into(),
            },
            sender_address: "sender".to_string(),
            destination_address: "contract".to_string(),
            references: vec![],
        };
        let payload = scan_payload(generic);
        assert_eq!(payload.transaction_type, TransferDataExtra::mock().transaction_type);
        assert_eq!(payload.website, Some(ApplicationMetadata::mock().url));
    }

    #[test]
    fn test_simulation_payload_only_for_utf8_payment_calls() {
        let mut extra = TransferDataExtra::mock();
        extra.data = Some(b"0xdeadbeef".to_vec());
        let mut metadata = ApplicationMetadata::mock();
        metadata.source = ApplicationMetadataSource::Payment;
        let generic = |metadata: ApplicationMetadata, extra: TransferDataExtra| GemTransactionInputType::Generic {
            asset: Asset::mock_sol(),
            metadata,
            extra: extra.into(),
        };

        assert_eq!(simulation_payload(&generic(metadata.clone(), extra.clone())), Some("0xdeadbeef".to_string()));

        let mut wallet_connect = metadata.clone();
        wallet_connect.source = ApplicationMetadataSource::WalletConnect;
        assert_eq!(simulation_payload(&generic(wallet_connect, extra.clone())), None);

        let mut binary = extra.clone();
        binary.data = Some(vec![0xff, 0xfe]);
        assert_eq!(simulation_payload(&generic(metadata.clone(), binary)), None);

        let mut empty = extra;
        empty.data = None;
        assert_eq!(simulation_payload(&generic(metadata, empty)), None);

        let swap = GemTransactionInputType::Swap {
            from_asset: Asset::mock_sol(),
            to_asset: Asset::mock_spl_token(),
            swap_data: SwapData::mock(),
        };
        assert_eq!(simulation_payload(&swap), None);
    }

    #[test]
    fn test_default_fee_priority_is_fast_only_for_bitcoin_swaps() {
        let bitcoin_swap = GemTransactionInputType::Swap {
            from_asset: Asset::from_chain(Chain::Bitcoin),
            to_asset: Asset::mock_sol(),
            swap_data: SwapData::mock(),
        };
        assert_eq!(default_fee_priority(bitcoin_swap), FeePriority::Fast.as_ref());
        let solana_swap = GemTransactionInputType::Swap {
            from_asset: Asset::mock_sol(),
            to_asset: Asset::mock_spl_token(),
            swap_data: SwapData::mock(),
        };
        assert_eq!(default_fee_priority(solana_swap), FeePriority::Normal.as_ref());
        assert_eq!(
            default_fee_priority(GemTransactionInputType::Transfer {
                asset: Asset::from_chain(Chain::Bitcoin)
            }),
            FeePriority::Normal.as_ref()
        );
    }

    #[test]
    fn test_insufficient_network_fee_only_for_empty_native_balances() {
        assert!(is_insufficient_network_fee(AssetId::from_chain(Chain::Ethereum), "0".into()));
        assert!(is_insufficient_network_fee(AssetId::from_chain(Chain::Ethereum), "".into()));
        assert!(!is_insufficient_network_fee(AssetId::from_chain(Chain::Ethereum), "10".into()));
        assert!(!is_insufficient_network_fee(AssetId::from_chain(Chain::Tron), "0".into()));
        assert!(!is_insufficient_network_fee(AssetId::from_chain(Chain::HyperCore), "0".into()));
        assert!(!is_insufficient_network_fee(
            AssetId::from(Chain::Ethereum, Some("0xdac17f958d2ee523a2206206994597c13d831ec7".into())),
            "0".into()
        ));
    }

    #[test]
    fn test_gateway_errors_keep_their_kind() {
        match load_error(GatewayError::NetworkError { msg: "timeout".to_string() }) {
            GemConfirmError::Network { msg } => assert_eq!(msg, "timeout"),
            error => panic!("expected a network error, got {error:?}"),
        }
        assert!(matches!(load_error(GatewayError::Offline), GemConfirmError::Offline));
        assert!(matches!(broadcast_error(vec![], GatewayError::Offline), GemConfirmError::Offline));
        assert!(matches!(broadcast_error(vec!["h1".to_string()], GatewayError::Offline), GemConfirmError::Broadcast { .. }));
        match load_error(GatewayError::PlatformError { msg: "dust".to_string() }) {
            GemConfirmError::Load { msg } => assert_eq!(msg, "Platform error: dust"),
            error => panic!("expected a load error, got {error:?}"),
        }
        match broadcast_error(vec![], GatewayError::NetworkError { msg: "offline".to_string() }) {
            GemConfirmError::Network { msg } => assert_eq!(msg, "offline"),
            error => panic!("expected a network error, got {error:?}"),
        }
        match broadcast_error(vec!["h1".to_string()], GatewayError::NetworkError { msg: "offline".to_string() }) {
            GemConfirmError::Broadcast { hashes, .. } => assert_eq!(hashes, vec!["h1".to_string()]),
            error => panic!("expected a partial broadcast error, got {error:?}"),
        }
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
