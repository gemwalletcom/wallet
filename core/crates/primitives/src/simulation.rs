use num_bigint::{BigInt, BigUint};
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_option_bigint_from_str, serialize_option_bigint};
use typeshare::typeshare;

use crate::{Asset, AssetId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationInput {
    #[serde(alias = "transaction")]
    pub encoded_transaction: String,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "pubkey", alias = "account", alias = "address")]
    pub signer_address: Option<String>,
}

impl SimulationInput {
    pub fn new(encoded_transaction: impl Into<String>) -> Self {
        Self {
            encoded_transaction: encoded_transaction.into(),
            signer_address: None,
        }
    }

    pub fn with_signer_address(mut self, signer_address: impl Into<String>) -> Self {
        self.signer_address = Some(signer_address.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SimulationSeverity {
    Low,
    Warning,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SimulationWarningApproval {
    pub asset_id: AssetId,
    #[serde(default, serialize_with = "serialize_option_bigint", deserialize_with = "deserialize_option_bigint_from_str")]
    pub value: Option<BigInt>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
pub enum SimulationWarningType {
    TokenApproval(SimulationWarningApproval),
    SuspiciousSpender,
    ExternallyOwnedSpender,
    NftCollectionApproval(AssetId),
    PermitApproval(SimulationWarningApproval),
    PermitBatchApproval(#[serde(serialize_with = "serialize_option_bigint", deserialize_with = "deserialize_option_bigint_from_str")] Option<BigInt>),
    ValidationError,
}

impl SimulationWarningType {
    fn requires_spender_verification(&self) -> bool {
        match self {
            Self::SuspiciousSpender | Self::ExternallyOwnedSpender | Self::ValidationError => false,
            Self::TokenApproval(_) | Self::NftCollectionApproval(_) | Self::PermitApproval(_) | Self::PermitBatchApproval(_) => true,
        }
    }

    fn approval_value(&self) -> Option<&Option<BigInt>> {
        match self {
            Self::TokenApproval(a) | Self::PermitApproval(a) => Some(&a.value),
            Self::PermitBatchApproval(value) => Some(value),
            Self::SuspiciousSpender | Self::ExternallyOwnedSpender | Self::NftCollectionApproval(_) | Self::ValidationError => None,
        }
    }

    fn collapse_priority(&self, severity: SimulationSeverity) -> u8 {
        match self {
            Self::ValidationError if severity == SimulationSeverity::Critical => 3,
            Self::ExternallyOwnedSpender => 2,
            _ if self.approval_value().is_some_and(Option::is_none) => 1,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SimulationWarning {
    pub severity: SimulationSeverity,
    pub warning: SimulationWarningType,
    pub message: Option<String>,
}

impl SimulationWarning {
    pub fn new(severity: SimulationSeverity, warning: SimulationWarningType, message: Option<String>) -> Self {
        Self { severity, warning, message }
    }

    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new(SimulationSeverity::Critical, SimulationWarningType::ValidationError, Some(message.into()))
    }

    pub fn execution_error(message: impl Into<String>) -> Self {
        Self::new(SimulationSeverity::Warning, SimulationWarningType::ValidationError, Some(message.into()))
    }

    fn collapse_priority(&self) -> u8 {
        self.warning.collapse_priority(self.severity)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SimulationBalanceChange {
    pub asset_id: AssetId,
    pub value: String,
    pub decimals: i32,
    pub name: Option<String>,
    pub symbol: Option<String>,
}

impl SimulationBalanceChange {
    pub fn new(asset_id: AssetId, value: BigInt) -> Self {
        Self {
            asset_id,
            value: value.to_string(),
            decimals: 0,
            name: None,
            symbol: None,
        }
    }

    pub fn with_asset(self, asset: Asset) -> Self {
        Self {
            name: Some(asset.name),
            symbol: Some(asset.symbol),
            decimals: asset.decimals,
            ..self
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SimulationPayloadFieldType {
    Text,
    Address,
    Timestamp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SimulationPayloadFieldDisplay {
    Primary,
    Secondary,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SimulationPayloadFieldKind {
    Contract,
    Method,
    Token,
    Spender,
    Value,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SimulationPayloadField {
    pub kind: SimulationPayloadFieldKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub value: String,
    pub field_type: SimulationPayloadFieldType,
    pub display: SimulationPayloadFieldDisplay,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SimulationHeader {
    pub asset_id: AssetId,
    #[serde(
        serialize_with = "serde_serializers::serialize_option_biguint",
        deserialize_with = "serde_serializers::deserialize_option_biguint_from_str"
    )]
    pub value: Option<BigUint>,
    pub is_unlimited: bool,
}

impl SimulationHeader {
    fn has_valid_value(&self) -> bool {
        self.is_unlimited || self.value.is_some()
    }
}

impl SimulationPayloadField {
    pub fn standard(kind: SimulationPayloadFieldKind, value: impl Into<String>, field_type: SimulationPayloadFieldType, display: SimulationPayloadFieldDisplay) -> Self {
        debug_assert!(kind != SimulationPayloadFieldKind::Custom);
        Self {
            kind,
            label: None,
            value: value.into(),
            field_type,
            display,
        }
    }

    pub fn custom(label: impl Into<String>, value: impl Into<String>, field_type: SimulationPayloadFieldType, display: SimulationPayloadFieldDisplay) -> Self {
        let label = label.into();
        debug_assert!(!label.is_empty());
        Self {
            kind: SimulationPayloadFieldKind::Custom,
            label: Some(label),
            value: value.into(),
            field_type,
            display,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SimulationResult {
    pub warnings: Vec<SimulationWarning>,
    pub balance_changes: Vec<SimulationBalanceChange>,
    pub payload: Vec<SimulationPayloadField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<SimulationHeader>,
}

impl SimulationResult {
    pub fn asset_ids(&self) -> Vec<AssetId> {
        let mut asset_ids = Vec::new();
        for asset_id in self
            .balance_changes
            .iter()
            .map(|change| &change.asset_id)
            .chain(self.header.as_ref().map(|header| &header.asset_id))
        {
            if !asset_ids.contains(asset_id) {
                asset_ids.push(asset_id.clone());
            }
        }
        asset_ids
    }

    pub fn valid_header(&self) -> Option<&SimulationHeader> {
        self.header.as_ref().filter(|header| header.has_valid_value())
    }

    pub fn new(warnings: Vec<SimulationWarning>, payload: Vec<SimulationPayloadField>) -> Self {
        Self {
            warnings: Self::collapse_warnings(warnings),
            balance_changes: vec![],
            payload: promote_single_secondary_payload_field(payload),
            header: None,
        }
    }

    pub fn prepend_warnings(mut self, warnings: Vec<SimulationWarning>) -> Self {
        self.warnings = Self::collapse_warnings(warnings.into_iter().chain(self.warnings).collect());
        self
    }

    pub fn requires_spender_verification(&self) -> bool {
        self.warnings.iter().any(|warning| warning.warning.requires_spender_verification())
    }

    fn collapse_warnings(warnings: Vec<SimulationWarning>) -> Vec<SimulationWarning> {
        let max_priority = warnings.iter().map(SimulationWarning::collapse_priority).max().unwrap_or(0);
        if max_priority > 0
            && let Some(warning) = warnings.iter().find(|warning| warning.collapse_priority() == max_priority).cloned()
        {
            return vec![warning];
        }

        warnings
    }
}

impl Default for SimulationResult {
    fn default() -> Self {
        Self::new(vec![], vec![])
    }
}

pub fn promote_single_secondary_payload_field(payload: Vec<SimulationPayloadField>) -> Vec<SimulationPayloadField> {
    let secondary_count = payload.iter().filter(|field| field.display == SimulationPayloadFieldDisplay::Secondary).count();

    if secondary_count != 1 {
        return payload;
    }

    payload
        .into_iter()
        .map(|field| {
            if field.display == SimulationPayloadFieldDisplay::Secondary {
                SimulationPayloadField {
                    display: SimulationPayloadFieldDisplay::Primary,
                    ..field
                }
            } else {
                field
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use num_bigint::{BigInt, BigUint};

    use super::{
        SimulationBalanceChange, SimulationHeader, SimulationInput, SimulationPayloadField, SimulationPayloadFieldDisplay, SimulationPayloadFieldKind, SimulationPayloadFieldType,
        SimulationResult, SimulationSeverity, SimulationWarning, SimulationWarningApproval, SimulationWarningType, promote_single_secondary_payload_field,
    };
    use crate::{AssetId, Chain, testkit::signer_mock::TEST_SOLANA_SENDER};

    #[test]
    fn test_asset_ids_cover_balance_changes_and_the_header() {
        let changed = AssetId::from_chain(Chain::Ethereum);
        let header = AssetId::from_chain(Chain::Solana);
        let simulation = SimulationResult {
            balance_changes: vec![
                SimulationBalanceChange {
                    asset_id: changed.clone(),
                    value: "1".to_string(),
                    name: None,
                    symbol: None,
                    decimals: 0,
                },
                SimulationBalanceChange {
                    asset_id: changed.clone(),
                    value: "2".to_string(),
                    name: None,
                    symbol: None,
                    decimals: 0,
                },
            ],
            header: Some(SimulationHeader {
                asset_id: header.clone(),
                value: Some(BigUint::from(2u32)),
                is_unlimited: false,
            }),
            ..SimulationResult::default()
        };

        assert_eq!(simulation.asset_ids(), vec![changed, header]);
        assert!(SimulationResult::default().asset_ids().is_empty());
    }

    #[test]
    fn test_valid_header_requires_a_readable_value() {
        let simulation = |value: Option<BigUint>, is_unlimited: bool| SimulationResult {
            header: Some(SimulationHeader {
                asset_id: AssetId::from_chain(Chain::Ethereum),
                value,
                is_unlimited,
            }),
            ..SimulationResult::default()
        };

        assert!(simulation(Some(BigUint::from(1000u32)), false).valid_header().is_some());
        assert!(simulation(None, true).valid_header().is_some());
        assert!(simulation(None, false).valid_header().is_none());
        assert!(SimulationResult::default().valid_header().is_none());
    }

    #[test]
    fn simulation_input_decodes_wallet_connect_transaction_field() {
        let input: SimulationInput = serde_json::from_value(serde_json::json!({
            "transaction": "AAACAAhkAAA",
        }))
        .unwrap();

        assert_eq!(input, SimulationInput::new("AAACAAhkAAA"));
    }

    #[test]
    fn simulation_input_decodes_optional_signer_aliases() {
        let input: SimulationInput = serde_json::from_value(serde_json::json!({
            "pubkey": TEST_SOLANA_SENDER,
            "transaction": "AAACAAhkAAA",
        }))
        .unwrap();

        assert_eq!(input, SimulationInput::new("AAACAAhkAAA").with_signer_address(TEST_SOLANA_SENDER));

        let input: SimulationInput = serde_json::from_value(serde_json::json!({
            "address": TEST_SOLANA_SENDER,
            "transaction": "AAACAAhkAAA",
        }))
        .unwrap();

        assert_eq!(input, SimulationInput::new("AAACAAhkAAA").with_signer_address(TEST_SOLANA_SENDER));
    }

    #[test]
    fn externally_owned_spender_warning_suppresses_secondary_approval_warning() {
        let result = SimulationResult::new(
            vec![
                SimulationWarning::new(
                    SimulationSeverity::Warning,
                    SimulationWarningType::PermitApproval(SimulationWarningApproval {
                        asset_id: "ethereum_0x123".into(),
                        value: Some(BigInt::from(100)),
                    }),
                    None,
                ),
                SimulationWarning::new(SimulationSeverity::Warning, SimulationWarningType::ExternallyOwnedSpender, None),
            ],
            Vec::<SimulationPayloadField>::new(),
        );

        assert_eq!(
            result.warnings,
            vec![SimulationWarning::new(SimulationSeverity::Warning, SimulationWarningType::ExternallyOwnedSpender, None,)]
        );
    }

    #[test]
    fn critical_validation_warning_suppresses_externally_owned_spender_warning() {
        let result = SimulationResult::new(
            vec![
                SimulationWarning::new(SimulationSeverity::Warning, SimulationWarningType::ExternallyOwnedSpender, None),
                SimulationWarning::new(
                    SimulationSeverity::Critical,
                    SimulationWarningType::ValidationError,
                    Some("Unable to verify spender is a contract".to_string()),
                ),
            ],
            vec![],
        );

        assert_eq!(
            result.warnings,
            vec![SimulationWarning::new(
                SimulationSeverity::Critical,
                SimulationWarningType::ValidationError,
                Some("Unable to verify spender is a contract".to_string()),
            )]
        );
    }

    #[test]
    fn approval_simulation_requires_spender_verification() {
        let result = SimulationResult::new(
            vec![SimulationWarning::new(
                SimulationSeverity::Warning,
                SimulationWarningType::PermitApproval(SimulationWarningApproval {
                    asset_id: "ethereum_0x123".into(),
                    value: Some(BigInt::from(100)),
                }),
                None,
            )],
            vec![],
        );

        assert!(result.requires_spender_verification());
    }

    #[test]
    fn validation_warning_suppresses_secondary_warnings() {
        let result = SimulationResult::new(
            vec![
                SimulationWarning::new(
                    SimulationSeverity::Warning,
                    SimulationWarningType::PermitApproval(SimulationWarningApproval {
                        asset_id: "ethereum_0x123".into(),
                        value: Some(BigInt::from(100)),
                    }),
                    None,
                ),
                SimulationWarning::new(
                    SimulationSeverity::Critical,
                    SimulationWarningType::ValidationError,
                    Some("Unable to verify spender is a contract".to_string()),
                ),
            ],
            Vec::<SimulationPayloadField>::new(),
        );

        assert_eq!(
            result.warnings,
            vec![SimulationWarning::new(
                SimulationSeverity::Critical,
                SimulationWarningType::ValidationError,
                Some("Unable to verify spender is a contract".to_string()),
            )]
        );
    }

    #[test]
    fn unlimited_warning_wins_when_present() {
        let result = SimulationResult::new(
            vec![SimulationWarning::new(
                SimulationSeverity::Warning,
                SimulationWarningType::PermitApproval(SimulationWarningApproval {
                    asset_id: "ethereum_0x123".into(),
                    value: None,
                }),
                None,
            )],
            Vec::<SimulationPayloadField>::new(),
        );

        assert_eq!(
            result.warnings,
            vec![SimulationWarning::new(
                SimulationSeverity::Warning,
                SimulationWarningType::PermitApproval(SimulationWarningApproval {
                    asset_id: "ethereum_0x123".into(),
                    value: None,
                }),
                None,
            )]
        );
    }

    #[test]
    fn unlimited_secondary_warning_suppresses_redundant_token_approval_warning() {
        let result = SimulationResult::new(
            vec![
                SimulationWarning::new(
                    SimulationSeverity::Warning,
                    SimulationWarningType::TokenApproval(SimulationWarningApproval {
                        asset_id: "ethereum_0x123".into(),
                        value: Some(BigInt::from(1000)),
                    }),
                    None,
                ),
                SimulationWarning::new(
                    SimulationSeverity::Warning,
                    SimulationWarningType::TokenApproval(SimulationWarningApproval {
                        asset_id: "ethereum_0x123".into(),
                        value: None,
                    }),
                    None,
                ),
            ],
            Vec::<SimulationPayloadField>::new(),
        );

        assert_eq!(
            result.warnings,
            vec![SimulationWarning::new(
                SimulationSeverity::Warning,
                SimulationWarningType::TokenApproval(SimulationWarningApproval {
                    asset_id: "ethereum_0x123".into(),
                    value: None,
                }),
                None,
            )]
        );
    }

    #[test]
    fn single_secondary_payload_field_is_promoted_to_primary() {
        let payload = promote_single_secondary_payload_field(vec![
            SimulationPayloadField::standard(
                SimulationPayloadFieldKind::Contract,
                "0x123",
                SimulationPayloadFieldType::Address,
                SimulationPayloadFieldDisplay::Primary,
            ),
            SimulationPayloadField::standard(
                SimulationPayloadFieldKind::Value,
                "Unlimited",
                SimulationPayloadFieldType::Text,
                SimulationPayloadFieldDisplay::Secondary,
            ),
        ]);

        assert_eq!(payload.len(), 2);
        assert!(payload.iter().all(|field| field.display == SimulationPayloadFieldDisplay::Primary));
    }

    #[test]
    fn multiple_secondary_payload_fields_stay_secondary() {
        let payload = promote_single_secondary_payload_field(vec![
            SimulationPayloadField::standard(
                SimulationPayloadFieldKind::Contract,
                "0x123",
                SimulationPayloadFieldType::Address,
                SimulationPayloadFieldDisplay::Primary,
            ),
            SimulationPayloadField::standard(
                SimulationPayloadFieldKind::Value,
                "Unlimited",
                SimulationPayloadFieldType::Text,
                SimulationPayloadFieldDisplay::Secondary,
            ),
            SimulationPayloadField::custom("expiration", "123", SimulationPayloadFieldType::Timestamp, SimulationPayloadFieldDisplay::Secondary),
        ]);

        assert_eq!(payload[1].display, SimulationPayloadFieldDisplay::Secondary);
        assert_eq!(payload[2].display, SimulationPayloadFieldDisplay::Secondary);
    }
}
