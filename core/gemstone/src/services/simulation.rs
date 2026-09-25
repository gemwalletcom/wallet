use std::collections::HashSet;
use std::sync::Arc;

use ::simulation::evm::SimulationClient;
use chain_traits::ChainSimulation;
use gem_evm::jsonrpc::TransactionObject;
use gem_evm::rpc::{EthereumClient, EthereumProvider};
use gem_jsonrpc::grpc::AlienGrpcTransport;
use gem_solana::rpc::{SolanaClient, SolanaProvider};
use gem_sui::rpc::{SuiClient, SuiProvider};
use gem_ton::rpc::client::TonClient;
use gem_tron::rpc::{TronProvider, client::TronClient};
use gem_wallet_connect::{SignDigestType as WcSignDigestType, WCEthereumTransactionData as WcEthereumTransactionData, WalletConnectTransactionType as WcWalletConnectTransactionType};
use primitives::{
    AddressName, AssetId, BlockExplorerLink, Chain, ChainAddress, EVMChain, SimulationInput, SimulationPayloadField, SimulationPayloadFieldKind, SimulationPayloadFieldType, SimulationResult, SimulationSeverity, SimulationWarning,
    SimulationWarningType,
};

use crate::models::copy::{GemCopy, address_copy};
use crate::models::custom_types::GemBigInt;
use crate::models::list::{GemListRow, GemListRowTitle, GemNoticeKind, suspicious_address_title};
use crate::services::localization::GemLocalizedText;
use crate::{
    GemstoneError,
    alien::{AlienClient, AlienProvider, AlienProviderWrapper, coalescing_provider, new_alien_client},
    message::sign_type::SignDigestType,
    network::JsonRpcClient,
    services::node::GemNodeService,
    wallet_connect::{WalletConnectTransactionType, simulation},
};

#[derive(uniffi::Object)]
pub struct GemSimulationService {
    provider: Arc<dyn AlienProvider>,
    nodes: Arc<GemNodeService>,
}

#[uniffi::export]
impl GemSimulationService {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>, nodes: Arc<GemNodeService>) -> Self {
        Self {
            provider: coalescing_provider(provider),
            nodes,
        }
    }
}

impl GemSimulationService {
    pub async fn simulate_sign_message(&self, chain: Chain, sign_type: SignDigestType, data: String, session_domain: String) -> Result<SimulationResult, GemstoneError> {
        let sign_type: WcSignDigestType = sign_type.into();
        let validation_warnings = simulation::sign_message_validation_warnings(chain, &sign_type, &data, &session_domain);

        let simulation = match sign_type {
            WcSignDigestType::Eip712 => match simulation::parse_eip712_message(&data) {
                Some(message) => self.simulate_eip712_message(chain, &message).await?,
                None => SimulationResult::default(),
            },
            _ => SimulationResult::default(),
        };

        Ok(simulation.prepend_warnings(validation_warnings))
    }

    /// Fails open, the way the scanner does: a provider that cannot answer reaches the review as an
    /// empty result rather than stopping a signature, and the validation warnings are unaffected.
    pub async fn simulate_send_transaction(&self, chain: Chain, transaction_type: WalletConnectTransactionType, data: String) -> Result<SimulationResult, GemstoneError> {
        let transaction_type: WcWalletConnectTransactionType = transaction_type.into();
        let validation_warnings = simulation::send_transaction_validation_warnings(&transaction_type, &data);

        let simulation = match &transaction_type {
            WcWalletConnectTransactionType::Ethereum => self.simulate_ethereum_transaction(chain, &data).await,
            WcWalletConnectTransactionType::Solana { .. } | WcWalletConnectTransactionType::Sui { .. } => self.simulate_encoded_transaction(&transaction_type, &data).await,
            WcWalletConnectTransactionType::Ton { .. } => self.simulate_chain_transaction(Chain::Ton, SimulationInput::new(&data)).await,
            WcWalletConnectTransactionType::Tron { .. } => self.simulate_chain_transaction(Chain::Tron, SimulationInput::new(&data)).await,
        }
        .unwrap_or_default();

        Ok(simulation.prepend_warnings(validation_warnings))
    }

    pub async fn simulate_transaction(&self, chain: Chain, encoded_transaction: String, signer_address: Option<String>) -> Result<SimulationResult, GemstoneError> {
        self.simulate_chain_transaction(chain, SimulationInput { encoded_transaction, signer_address }).await
    }
}

impl GemSimulationService {
    async fn simulate_eip712_message(&self, chain: Chain, message: &gem_evm::eip712::EIP712Message) -> Result<SimulationResult, GemstoneError> {
        let provider = self.ethereum_provider(chain)?;
        Ok(SimulationClient::new(&provider).simulate_eip712_message(chain, message).await?)
    }

    async fn simulate_ethereum_transaction(&self, chain: Chain, data: &str) -> Result<SimulationResult, GemstoneError> {
        let transaction = simulation::decode_ethereum_transaction(data)?;
        let calldata = simulation::decode_ethereum_calldata(&transaction);
        let provider = self.ethereum_provider(chain)?;

        if ::simulation::evm::is_approval(chain, &calldata, &transaction.to) {
            return Ok(SimulationClient::new(&provider).simulate_evm_calldata(chain, &calldata, &transaction.to).await?);
        }

        let (calldata_result, balance_result) = self.simulate_calldata_and_balance_changes(chain, &calldata, &provider, &transaction).await;
        let calldata_result = calldata_result?;
        let balance_result = balance_result.unwrap_or_default();

        Ok(SimulationResult {
            balance_changes: balance_result.balance_changes,
            ..calldata_result
        }
        .prepend_warnings(balance_result.warnings))
    }

    async fn simulate_calldata_and_balance_changes(
        &self,
        chain: Chain,
        calldata: &[u8],
        provider: &EthereumProvider<AlienClient>,
        transaction: &WcEthereumTransactionData,
    ) -> (Result<SimulationResult, GemstoneError>, Result<SimulationResult, GemstoneError>) {
        let calldata_task = async {
            if calldata.is_empty() {
                Ok(SimulationResult::default())
            } else {
                SimulationClient::new(provider).simulate_evm_calldata(chain, calldata, &transaction.to).await.map_err(GemstoneError::from)
            }
        };
        futures::join!(calldata_task, self.simulate_ethereum_balance_changes(provider, transaction))
    }

    async fn simulate_ethereum_balance_changes(&self, provider: &EthereumProvider<AlienClient>, transaction: &WcEthereumTransactionData) -> Result<SimulationResult, GemstoneError> {
        let encoded_transaction = serde_json::to_string(&map_transaction_object(transaction)).map_err(|error| error.to_string())?;

        Ok(provider.simulate_transaction(SimulationInput::new(encoded_transaction)).await?)
    }

    async fn simulate_encoded_transaction(&self, transaction_type: &WcWalletConnectTransactionType, data: &str) -> Result<SimulationResult, GemstoneError> {
        let chain = match transaction_type {
            WcWalletConnectTransactionType::Solana { .. } => Chain::Solana,
            WcWalletConnectTransactionType::Sui { .. } => Chain::Sui,
            _ => return Err("Chain does not use encoded transaction simulation".into()),
        };
        let input: SimulationInput = serde_json::from_str(data).map_err(|error| error.to_string())?;
        self.simulate_chain_transaction(chain, input).await
    }

    async fn simulate_chain_transaction(&self, chain: Chain, input: SimulationInput) -> Result<SimulationResult, GemstoneError> {
        Ok(self.chain_simulation(chain)?.simulate_transaction(input).await?)
    }

    fn ethereum_provider(&self, chain: Chain) -> Result<EthereumProvider<AlienClient>, GemstoneError> {
        let chain = EVMChain::from_chain(chain).ok_or_else(|| format!("{chain} is not an EVM chain"))?;
        let url = self.nodes.node_url(chain.to_chain());
        let client = new_alien_client(url, self.provider.clone());
        Ok(EthereumProvider::new_rpc_only(EthereumClient::new(JsonRpcClient::new(client), chain)))
    }

    fn chain_simulation(&self, chain: Chain) -> Result<Box<dyn ChainSimulation>, GemstoneError> {
        let url = self.nodes.node_url(chain);
        let new_client = || new_alien_client(url.clone(), self.provider.clone());
        match chain {
            Chain::Solana => Ok(Box::new(SolanaProvider::new_rpc_only(SolanaClient::new(JsonRpcClient::new(new_client()))))),
            Chain::Sui => {
                let transport = AlienGrpcTransport::new(Arc::new(AlienProviderWrapper::new(self.provider.clone())));
                Ok(Box::new(SuiProvider::new_rpc_only(SuiClient::new_with_transport(url, Arc::new(transport)))))
            }
            Chain::Ton => Ok(Box::new(TonClient::new(new_client()))),
            Chain::Tron => Ok(Box::new(TronProvider::new_rpc_only(TronClient::new(new_client())))),
            _ => Err(format!("{chain} does not support transaction simulation").into()),
        }
    }
}

/// Keeps the gas limit so out-of-gas failures surface, but omits fee prices - they make the trace charge gas and leak fee accounting into the signer's balance diff.
fn map_transaction_object(transaction: &WcEthereumTransactionData) -> TransactionObject {
    TransactionObject {
        from: Some(transaction.from.clone()),
        to: transaction.to.clone(),
        gas: transaction.gas.clone().or_else(|| transaction.gas_limit.clone()),
        gas_price: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
        value: transaction.value.clone(),
        data: transaction.data.clone().unwrap_or_default(),
    }
}

#[derive(Default)]
pub struct GemSimulationFormatter {}

impl GemSimulationFormatter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn payload_fields(&self, payload: Vec<SimulationPayloadField>, shows_header: bool) -> Vec<SimulationPayloadField> {
        if !shows_header {
            return payload;
        }
        payload.into_iter().filter(|field| field.kind != SimulationPayloadFieldKind::Value && field.kind != SimulationPayloadFieldKind::Token).collect()
    }

    pub fn shows_header(&self, simulation: Option<SimulationResult>, is_approval: bool) -> bool {
        is_approval || simulation.as_ref().and_then(SimulationResult::valid_header).is_some()
    }

    pub fn balance_changes(&self, simulation: Option<SimulationResult>, known_asset_ids: Vec<AssetId>) -> Vec<GemSimulationChange> {
        let known: HashSet<String> = known_asset_ids.into_iter().map(|asset_id| asset_id.to_string()).collect();
        simulation
            .map(|simulation| simulation.balance_changes)
            .unwrap_or_default()
            .into_iter()
            .filter(|change| known.contains(&change.asset_id.to_string()))
            .filter_map(|change| {
                (change.value != GemBigInt::ZERO).then_some(GemSimulationChange {
                    asset_id: change.asset_id,
                    value: change.value,
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum GemSimulationPayloadTitle {
    Contract,
    Method,
    Token,
    Spender,
    Value,
    Expiration,
    Custom { label: String },
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemSimulationPayloadValue {
    Text { text: String },
    Address { display: String, copy: GemCopy, explorer: BlockExplorerLink },
    Timestamp { unix_ms: i64 },
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemSimulationPayloadRow {
    pub title: GemSimulationPayloadTitle,
    pub value: GemSimulationPayloadValue,
}

pub fn payload_rows(fields: &[SimulationPayloadField], chain: Chain, address_url: impl Fn(Chain, String) -> BlockExplorerLink) -> Vec<GemSimulationPayloadRow> {
    fields.iter().map(|field| payload_row(field, chain, &address_url)).collect()
}

pub fn named_payload_rows(rows: Vec<GemSimulationPayloadRow>, names: &[AddressName]) -> Vec<GemSimulationPayloadRow> {
    rows.into_iter()
        .map(|row| GemSimulationPayloadRow {
            value: match row.value {
                GemSimulationPayloadValue::Address { copy, explorer, .. } => address_value(copy, names, explorer),
                GemSimulationPayloadValue::Text { .. } | GemSimulationPayloadValue::Timestamp { .. } => row.value,
            },
            title: row.title,
        })
        .collect()
}

pub fn address_requests(rows: &[GemSimulationPayloadRow], chain: Chain) -> Vec<ChainAddress> {
    rows.iter()
        .filter_map(|row| match &row.value {
            GemSimulationPayloadValue::Address { copy, .. } => Some(ChainAddress::new(chain, copy.value.clone())),
            GemSimulationPayloadValue::Text { .. } | GemSimulationPayloadValue::Timestamp { .. } => None,
        })
        .collect()
}

fn payload_row(field: &SimulationPayloadField, chain: Chain, address_url: impl Fn(Chain, String) -> BlockExplorerLink) -> GemSimulationPayloadRow {
    GemSimulationPayloadRow {
        title: match field.kind {
            SimulationPayloadFieldKind::Contract => GemSimulationPayloadTitle::Contract,
            SimulationPayloadFieldKind::Method => GemSimulationPayloadTitle::Method,
            SimulationPayloadFieldKind::Token => GemSimulationPayloadTitle::Token,
            SimulationPayloadFieldKind::Spender => GemSimulationPayloadTitle::Spender,
            SimulationPayloadFieldKind::Value => GemSimulationPayloadTitle::Value,
            SimulationPayloadFieldKind::Expiration => GemSimulationPayloadTitle::Expiration,
            SimulationPayloadFieldKind::Custom => GemSimulationPayloadTitle::Custom {
                label: field.label.clone().unwrap_or_default(),
            },
        },
        value: match field.field_type {
            SimulationPayloadFieldType::Text => GemSimulationPayloadValue::Text { text: field.value.clone() },
            SimulationPayloadFieldType::Address => address_value(address_copy(chain, field.value.clone()), &[], address_url(chain, field.value.clone())),
            SimulationPayloadFieldType::Timestamp => match timestamp_unix_ms(&field.value) {
                Some(unix_ms) => GemSimulationPayloadValue::Timestamp { unix_ms },
                None => GemSimulationPayloadValue::Text { text: field.value.clone() },
            },
        },
    }
}

fn address_value(copy: GemCopy, names: &[AddressName], explorer: BlockExplorerLink) -> GemSimulationPayloadValue {
    let display = names
        .iter()
        .find(|name| name.address.eq_ignore_ascii_case(&copy.value) && !name.name.is_empty() && !name.name.eq_ignore_ascii_case(&copy.value))
        .map(|name| format!("{} ({})", name.name, copy.display))
        .unwrap_or_else(|| copy.display.clone());
    GemSimulationPayloadValue::Address { display, copy, explorer }
}

fn timestamp_unix_ms(value: &str) -> Option<i64> {
    if let Ok(seconds) = value.parse::<f64>() {
        return Some((seconds * 1000.0) as i64);
    }
    chrono::DateTime::parse_from_rfc3339(value).ok().map(|date| date.timestamp_millis())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WarningKind {
    UnlimitedApproval,
    NftCollectionApproval,
    ExternallyOwnedSpender,
    SuspiciousSpender,
    ValidationError,
}

impl WarningKind {
    fn title(self, severity: SimulationSeverity) -> GemListRowTitle {
        match self {
            Self::UnlimitedApproval => GemListRowTitle::UnlimitedApproval,
            Self::NftCollectionApproval => GemListRowTitle::NftCollectionApproval,
            Self::ExternallyOwnedSpender => GemListRowTitle::Warning,
            Self::SuspiciousSpender => suspicious_address_title(notice_kind(severity)),
            Self::ValidationError => match severity {
                SimulationSeverity::Critical => GemListRowTitle::Error,
                SimulationSeverity::Low | SimulationSeverity::Warning => GemListRowTitle::Warning,
            },
        }
    }

    fn default_message(self, severity: SimulationSeverity) -> Option<GemLocalizedText> {
        match self {
            Self::UnlimitedApproval => Some(GemLocalizedText::UnlimitedApprovalWarning),
            Self::ExternallyOwnedSpender => Some(GemLocalizedText::ExternallyOwnedSpenderWarning),
            Self::SuspiciousSpender => Some(GemLocalizedText::SuspiciousAddressDescription),
            Self::NftCollectionApproval => None,
            Self::ValidationError => match severity {
                SimulationSeverity::Critical => Some(GemLocalizedText::ErrorOccurred),
                SimulationSeverity::Low | SimulationSeverity::Warning => None,
            },
        }
    }
}

fn notice_kind(severity: SimulationSeverity) -> GemNoticeKind {
    match severity {
        SimulationSeverity::Critical => GemNoticeKind::Error,
        SimulationSeverity::Low | SimulationSeverity::Warning => GemNoticeKind::Warning,
    }
}

pub fn warning_rows(warnings: &[SimulationWarning]) -> Vec<GemListRow> {
    warnings
        .iter()
        .filter_map(|warning| {
            let kind = match &warning.warning {
                SimulationWarningType::TokenApproval(approval) | SimulationWarningType::PermitApproval(approval) => approval.value.is_none().then_some(WarningKind::UnlimitedApproval),
                SimulationWarningType::PermitBatchApproval(value) => value.is_none().then_some(WarningKind::UnlimitedApproval),
                SimulationWarningType::NftCollectionApproval(_) => Some(WarningKind::NftCollectionApproval),
                SimulationWarningType::ExternallyOwnedSpender => Some(WarningKind::ExternallyOwnedSpender),
                SimulationWarningType::SuspiciousSpender => Some(WarningKind::SuspiciousSpender),
                SimulationWarningType::ValidationError => Some(WarningKind::ValidationError),
            }?;
            Some(GemListRow::Notice {
                title: kind.title(warning.severity),
                message: warning.message.clone().map(|text| GemLocalizedText::Text { text }).or_else(|| kind.default_message(warning.severity)),
                kind: notice_kind(warning.severity),
            })
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct GemSimulationChange {
    pub asset_id: AssetId,
    pub value: GemBigInt,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::list::suspicious_address_notice;
    use crate::services::node::GemNodeService;
    use crate::testkit::{TestAlienProvider, mock_wc_ethereum_transaction_data};
    use num_bigint::BigInt;
    use primitives::{AddressType, SimulationBalanceChange, SimulationPayloadFieldDisplay, SimulationWarningApproval, VerificationStatus};

    #[test]
    fn test_warning_rows_hide_bounded_approvals_and_keep_every_other_warning() {
        let rows = warning_rows(&[
            SimulationWarning::mock(SimulationWarningType::TokenApproval(SimulationWarningApproval::mock(Some(BigInt::from(1))))),
            SimulationWarning::mock(SimulationWarningType::PermitApproval(SimulationWarningApproval::mock(Some(BigInt::from(1))))),
            SimulationWarning::mock(SimulationWarningType::PermitBatchApproval(Some(BigInt::from(1)))),
            SimulationWarning::mock(SimulationWarningType::TokenApproval(SimulationWarningApproval::mock(None))),
            SimulationWarning::mock(SimulationWarningType::PermitApproval(SimulationWarningApproval::mock(None))),
            SimulationWarning::mock(SimulationWarningType::PermitBatchApproval(None)),
            SimulationWarning::mock(SimulationWarningType::NftCollectionApproval(AssetId::from_chain(Chain::Ethereum))),
            SimulationWarning::mock(SimulationWarningType::ExternallyOwnedSpender),
            SimulationWarning::mock(SimulationWarningType::SuspiciousSpender),
            SimulationWarning::validation_error("Chain ID mismatch"),
        ]);
        let notice = |title: GemListRowTitle, message: Option<GemLocalizedText>| GemListRow::Notice {
            title,
            message,
            kind: GemNoticeKind::Warning,
        };
        let unlimited = notice(GemListRowTitle::UnlimitedApproval, Some(GemLocalizedText::UnlimitedApprovalWarning));

        assert_eq!(
            rows,
            vec![
                unlimited.clone(),
                unlimited.clone(),
                unlimited,
                notice(GemListRowTitle::NftCollectionApproval, None),
                notice(GemListRowTitle::Warning, Some(GemLocalizedText::ExternallyOwnedSpenderWarning)),
                suspicious_address_notice(GemNoticeKind::Warning),
                GemListRow::Notice {
                    title: GemListRowTitle::Error,
                    message: Some(GemLocalizedText::Text { text: "Chain ID mismatch".to_string() }),
                    kind: GemNoticeKind::Error,
                },
            ]
        );
    }

    #[test]
    fn test_a_warning_without_its_own_message_reads_the_kind_default() {
        let critical = SimulationWarning {
            message: None,
            ..SimulationWarning::validation_error("")
        };
        assert_eq!(
            warning_rows(&[critical]),
            vec![GemListRow::Notice {
                title: GemListRowTitle::Error,
                message: Some(GemLocalizedText::ErrorOccurred),
                kind: GemNoticeKind::Error,
            }]
        );
        let low = SimulationWarning {
            message: None,
            severity: SimulationSeverity::Warning,
            ..SimulationWarning::validation_error("")
        };
        assert_eq!(
            warning_rows(&[low]),
            vec![GemListRow::Notice {
                title: GemListRowTitle::Warning,
                message: None,
                kind: GemNoticeKind::Warning,
            }]
        );
    }

    #[test]
    fn test_balance_changes_drop_zero_and_unknown_assets() {
        let simulation = SimulationResult {
            balance_changes: vec![
                SimulationBalanceChange::mock(AssetId::new("ethereum").unwrap(), BigInt::from(-1000), 18),
                SimulationBalanceChange::mock(AssetId::new("ethereum_0xdac17f958d2ee523a2206206994597c13d831ec7").unwrap(), BigInt::from(2000), 18),
                SimulationBalanceChange::mock(AssetId::new("solana").unwrap(), BigInt::from(0), 18),
                SimulationBalanceChange::mock(AssetId::new("doge").unwrap(), BigInt::from(500), 18),
            ],
            ..SimulationResult::default()
        };
        let known = ["ethereum", "ethereum_0xdac17f958d2ee523a2206206994597c13d831ec7", "solana"].into_iter().map(|id| AssetId::new(id).unwrap()).collect();

        let formatter = GemSimulationFormatter::new();
        let changes = formatter.balance_changes(Some(simulation), known);

        assert_eq!(
            changes.iter().map(|change| change.value.to_string()).collect::<Vec<_>>(),
            vec!["-1000", "2000"],
            "keeps signed non-zero changes for known assets only"
        );
        assert!(formatter.balance_changes(None, vec![]).is_empty());
    }

    #[test]
    fn test_map_transaction_object_normalizes_empty_calldata_to_0x() {
        let transaction_object = map_transaction_object(&mock_wc_ethereum_transaction_data());

        assert_eq!(serde_json::to_value(transaction_object).unwrap()["data"], "0x");
    }

    #[test]
    fn test_map_transaction_object_passes_gas_limit_and_omits_fee_prices() {
        let transaction = WcEthereumTransactionData {
            gas_limit: Some("0x5208".to_string()),
            gas_price: Some("0x9502f900".to_string()),
            max_fee_per_gas: Some("0x59682f10".to_string()),
            max_priority_fee_per_gas: Some("0x3b9aca00".to_string()),
            ..mock_wc_ethereum_transaction_data()
        };

        let transaction_object = map_transaction_object(&transaction);

        assert_eq!(transaction_object.gas.as_deref(), Some("0x5208"));
        assert_eq!(transaction_object.gas_price, None);
        assert_eq!(transaction_object.max_fee_per_gas, None);
        assert_eq!(transaction_object.max_priority_fee_per_gas, None);
    }

    #[test]
    fn test_the_value_and_token_fields_give_way_to_the_header() {
        let payload = vec![
            SimulationPayloadField::standard(SimulationPayloadFieldKind::Value, "", SimulationPayloadFieldType::Text, SimulationPayloadFieldDisplay::Primary),
            SimulationPayloadField::standard(SimulationPayloadFieldKind::Token, "", SimulationPayloadFieldType::Text, SimulationPayloadFieldDisplay::Primary),
            SimulationPayloadField::standard(SimulationPayloadFieldKind::Spender, "", SimulationPayloadFieldType::Text, SimulationPayloadFieldDisplay::Primary),
        ];
        let formatter = GemSimulationFormatter::new();

        assert_eq!(formatter.payload_fields(payload.clone(), false).len(), 3);
        assert_eq!(formatter.payload_fields(payload, true).into_iter().map(|field| field.kind).collect::<Vec<_>>(), vec![SimulationPayloadFieldKind::Spender]);
    }

    fn link(chain: Chain, address: String) -> BlockExplorerLink {
        BlockExplorerLink {
            name: "Etherscan".to_string(),
            link: format!("https://etherscan.io/address/{address}?{chain}"),
        }
    }

    #[test]
    fn test_payload_rows_title_by_kind_and_carry_the_copy_the_link_or_the_parsed_timestamp() {
        let address = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4";
        let fields = vec![
            SimulationPayloadField::standard(SimulationPayloadFieldKind::Spender, address, SimulationPayloadFieldType::Address, SimulationPayloadFieldDisplay::Primary),
            SimulationPayloadField::standard(SimulationPayloadFieldKind::Expiration, "1662714817", SimulationPayloadFieldType::Timestamp, SimulationPayloadFieldDisplay::Primary),
            SimulationPayloadField::custom("issuedAt", "2024-01-02T03:04:05.123Z", SimulationPayloadFieldType::Timestamp, SimulationPayloadFieldDisplay::Secondary),
            SimulationPayloadField::custom("statement", "Sign in", SimulationPayloadFieldType::Text, SimulationPayloadFieldDisplay::Secondary),
        ];
        let copy = address_copy(Chain::Ethereum, address.to_string());

        let rows = payload_rows(&fields, Chain::Ethereum, link);

        assert_ne!(copy.display, address, "the row shows the short address");
        assert_eq!(
            rows,
            vec![
                GemSimulationPayloadRow {
                    title: GemSimulationPayloadTitle::Spender,
                    value: GemSimulationPayloadValue::Address {
                        display: copy.display.clone(),
                        copy,
                        explorer: link(Chain::Ethereum, address.to_string()),
                    },
                },
                GemSimulationPayloadRow {
                    title: GemSimulationPayloadTitle::Expiration,
                    value: GemSimulationPayloadValue::Timestamp { unix_ms: 1_662_714_817_000 },
                },
                GemSimulationPayloadRow {
                    title: GemSimulationPayloadTitle::Custom { label: "issuedAt".to_string() },
                    value: GemSimulationPayloadValue::Timestamp { unix_ms: 1_704_164_645_123 },
                },
                GemSimulationPayloadRow {
                    title: GemSimulationPayloadTitle::Custom { label: "statement".to_string() },
                    value: GemSimulationPayloadValue::Text { text: "Sign in".to_string() },
                },
            ]
        );
    }

    #[test]
    fn test_named_payload_rows_prefix_a_known_name_skip_names_that_repeat_the_address_and_keep_the_link() {
        let address = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4";
        let other = "0x0000000000000000000000000000000000000001";
        let rows = payload_rows(
            &[
                SimulationPayloadField::standard(SimulationPayloadFieldKind::Spender, address, SimulationPayloadFieldType::Address, SimulationPayloadFieldDisplay::Primary),
                SimulationPayloadField::standard(SimulationPayloadFieldKind::Contract, other, SimulationPayloadFieldType::Address, SimulationPayloadFieldDisplay::Primary),
            ],
            Chain::Ethereum,
            link,
        );
        let name = |address: &str, name: &str| AddressName {
            chain: Chain::Ethereum,
            address: address.to_string(),
            name: name.to_string(),
            address_type: AddressType::Contract,
            status: VerificationStatus::Verified,
            image_url: None,
        };

        let named = named_payload_rows(rows, &[name(&address.to_lowercase(), "Hyperliquid"), name(other, other)]);

        let copy = address_copy(Chain::Ethereum, address.to_string());
        assert_eq!(
            named[0].value,
            GemSimulationPayloadValue::Address {
                display: format!("Hyperliquid ({})", copy.display),
                copy,
                explorer: link(Chain::Ethereum, address.to_string()),
            }
        );
        let other_copy = address_copy(Chain::Ethereum, other.to_string());
        assert_eq!(
            named[1].value,
            GemSimulationPayloadValue::Address {
                display: other_copy.display.clone(),
                copy: other_copy,
                explorer: link(Chain::Ethereum, other.to_string()),
            }
        );
        assert_eq!(
            address_requests(&named, Chain::Ethereum),
            vec![ChainAddress::new(Chain::Ethereum, address.to_string()), ChainAddress::new(Chain::Ethereum, other.to_string())]
        );
    }

    #[test]
    fn test_wallet_connect_send_transaction_ignores_simulation_error() {
        futures::executor::block_on(async {
            let provider = Arc::new(TestAlienProvider::with_status(200));
            let service = GemSimulationService::new(provider.clone(), Arc::new(GemNodeService::mock()));

            let result = service
                .simulate_send_transaction(
                    Chain::Solana,
                    WalletConnectTransactionType::Solana {
                        output_type: primitives::TransferDataOutputType::EncodedTransaction,
                    },
                    "invalid simulation input".to_string(),
                )
                .await;

            assert!(result.is_ok());
            assert!(provider.requested_paths().is_empty());
        });
    }
}
