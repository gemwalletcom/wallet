use gem_solana::siws::SiwsMessage;
use primitives::{SimulationPayloadField, SimulationPayloadFieldDisplay, SimulationPayloadFieldKind, SimulationPayloadFieldType, promote_single_secondary_payload_field};
use std::borrow::Cow;
use std::collections::HashSet;

use crate::{
    message::eip712::{GemEIP712Message, GemEIP712Value, GemEIP712ValueType},
    message::sign_type::MessageType,
    siwe::SiweMessage,
};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct MessagePayloadPreview {
    pub message_type: MessageType,
    pub primary: Vec<SimulationPayloadField>,
    pub secondary: Vec<SimulationPayloadField>,
}

#[derive(Debug, Clone, PartialEq)]
struct MessagePayloadField {
    kind: Option<SimulationPayloadFieldKind>,
    label: Option<String>,
    value: String,
    field_type: SimulationPayloadFieldType,
    display: SimulationPayloadFieldDisplay,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum PayloadMergeKey {
    Kind(SimulationPayloadFieldKind),
    Label(String),
}

enum CanonicalPayloadLabel<'a> {
    Kind(SimulationPayloadFieldKind),
    Custom(Cow<'a, str>),
}

impl GemEIP712Message {
    pub(super) fn payload_preview(&self, simulation_payload: Vec<SimulationPayloadField>) -> MessagePayloadPreview {
        grouped_payload_preview(MessageType::Eip712, eip712_preview_fields(self), simulation_payload)
    }
}

impl MessagePayloadPreview {
    pub(super) fn from_siwe(message: &SiweMessage, simulation_payload: Vec<SimulationPayloadField>) -> Self {
        grouped_payload_preview(MessageType::Siwe, siwe_preview_fields(message), simulation_payload)
    }

    pub(super) fn from_siws(message: &SiwsMessage, simulation_payload: Vec<SimulationPayloadField>) -> Self {
        let mut fields = vec![
            MessagePayloadField::custom("domain", message.domain.clone(), SimulationPayloadFieldType::Text, SimulationPayloadFieldDisplay::Secondary),
            MessagePayloadField::custom(
                "address",
                message.address.clone(),
                SimulationPayloadFieldType::Address,
                SimulationPayloadFieldDisplay::Secondary,
            ),
        ];
        fields.extend(
            [
                ("statement", message.statement.as_ref(), SimulationPayloadFieldType::Text),
                ("uri", message.uri.as_ref(), SimulationPayloadFieldType::Text),
                ("version", message.version.as_ref(), SimulationPayloadFieldType::Text),
                ("chainId", message.chain_id.as_ref(), SimulationPayloadFieldType::Text),
                ("nonce", message.nonce.as_ref(), SimulationPayloadFieldType::Text),
                ("issuedAt", message.issued_at.as_ref(), SimulationPayloadFieldType::Timestamp),
                ("expirationTime", message.expiration_time.as_ref(), SimulationPayloadFieldType::Timestamp),
                ("notBefore", message.not_before.as_ref(), SimulationPayloadFieldType::Timestamp),
                ("requestId", message.request_id.as_ref(), SimulationPayloadFieldType::Text),
            ]
            .into_iter()
            .filter_map(|(label, value, field_type)| value.map(|value| MessagePayloadField::custom(label, value.clone(), field_type, SimulationPayloadFieldDisplay::Secondary))),
        );
        if !message.resources.is_empty() {
            fields.push(MessagePayloadField::custom(
                "resources",
                message.resources.join("\n"),
                SimulationPayloadFieldType::Text,
                SimulationPayloadFieldDisplay::Secondary,
            ));
        }
        grouped_payload_preview(MessageType::Siws, fields, simulation_payload)
    }
}

fn grouped_payload_preview(message_type: MessageType, preview_fields: Vec<MessagePayloadField>, simulation_payload: Vec<SimulationPayloadField>) -> MessagePayloadPreview {
    let merged_payload = merge_payload(simulation_payload.clone(), preview_fields);
    let grouped_payload = if simulation_payload.is_empty() {
        apply_preview_display_grouping(merged_payload)
    } else {
        merged_payload
    };

    let grouped_payload = promote_single_secondary_payload_field(grouped_payload);
    let grouped_payload = promote_secondary_payload_when_primary_is_empty(grouped_payload);

    MessagePayloadPreview {
        message_type,
        primary: grouped_payload
            .iter()
            .filter(|field| field.display == SimulationPayloadFieldDisplay::Primary)
            .cloned()
            .collect(),
        secondary: grouped_payload
            .iter()
            .filter(|field| field.display == SimulationPayloadFieldDisplay::Secondary)
            .cloned()
            .collect(),
    }
}

fn promote_secondary_payload_when_primary_is_empty(payload: Vec<SimulationPayloadField>) -> Vec<SimulationPayloadField> {
    if payload.iter().any(|field| field.display == SimulationPayloadFieldDisplay::Primary) {
        return payload;
    }

    payload
        .into_iter()
        .map(|field| SimulationPayloadField {
            display: SimulationPayloadFieldDisplay::Primary,
            ..field
        })
        .collect()
}

fn merge_payload(mut simulation_payload: Vec<SimulationPayloadField>, preview_fields: Vec<MessagePayloadField>) -> Vec<SimulationPayloadField> {
    let mut seen_field_keys: HashSet<_> = simulation_payload.iter().map(payload_merge_key).collect();

    for field in preview_fields.into_iter().filter(|field| !field.value.is_empty()) {
        let merge_key = field.merge_key();
        if !seen_field_keys.insert(merge_key) {
            continue;
        }
        simulation_payload.push(field.into_public_field());
    }

    simulation_payload
}

fn apply_preview_display_grouping(payload: Vec<SimulationPayloadField>) -> Vec<SimulationPayloadField> {
    let primary_keys = preview_primary_keys(&payload);

    payload
        .into_iter()
        .map(|field| SimulationPayloadField {
            display: if primary_keys.contains(&payload_merge_key(&field)) {
                SimulationPayloadFieldDisplay::Primary
            } else {
                SimulationPayloadFieldDisplay::Secondary
            },
            ..field
        })
        .collect()
}

fn preview_primary_keys(payload: &[SimulationPayloadField]) -> HashSet<PayloadMergeKey> {
    let keys: HashSet<_> = payload.iter().map(payload_merge_key).collect();
    let has_contract_action_payload = keys.iter().any(PayloadMergeKey::is_contract_action_key);

    if has_contract_action_payload {
        let mut primary_keys = [
            PayloadMergeKey::Kind(SimulationPayloadFieldKind::Contract),
            PayloadMergeKey::Kind(SimulationPayloadFieldKind::Method),
        ]
        .into_iter()
        .filter(|key| keys.contains(key))
        .collect::<HashSet<_>>();

        if keys.contains(&PayloadMergeKey::Kind(SimulationPayloadFieldKind::Token)) {
            primary_keys.insert(PayloadMergeKey::Kind(SimulationPayloadFieldKind::Token));
        } else if keys.contains(&PayloadMergeKey::Kind(SimulationPayloadFieldKind::Spender)) {
            primary_keys.insert(PayloadMergeKey::Kind(SimulationPayloadFieldKind::Spender));
        }

        return primary_keys;
    }

    [PayloadMergeKey::Label("domain".to_string()), PayloadMergeKey::Label("address".to_string())]
        .into_iter()
        .filter(|key| keys.contains(key))
        .collect()
}

fn eip712_preview_fields(message: &GemEIP712Message) -> Vec<MessagePayloadField> {
    let mut fields = vec![primary_type_payload_field(message)];

    if let Some(domain_name) = message.domain.name.as_ref() {
        fields.insert(
            0,
            MessagePayloadField::custom("domain", domain_name.clone(), SimulationPayloadFieldType::Text, SimulationPayloadFieldDisplay::Secondary),
        );
    }

    if let Some(verifying_contract) = message.domain.verifying_contract.as_ref() {
        fields.push(MessagePayloadField::standard(
            SimulationPayloadFieldKind::Contract,
            verifying_contract.clone(),
            SimulationPayloadFieldType::Address,
            SimulationPayloadFieldDisplay::Secondary,
        ));
    }

    fields.extend(message.message.iter().flat_map(|section| section.values.iter().map(payload_field_from_eip712_value)));
    fields
}

fn primary_type_payload_field(message: &GemEIP712Message) -> MessagePayloadField {
    let primary_type = message.message.first().map(|section| section.name.clone()).unwrap_or_default();

    MessagePayloadField::standard(
        SimulationPayloadFieldKind::Method,
        primary_type,
        SimulationPayloadFieldType::Text,
        SimulationPayloadFieldDisplay::Secondary,
    )
}

fn payload_field_from_eip712_value(field: &GemEIP712Value) -> MessagePayloadField {
    let field_type = match field.value_type {
        GemEIP712ValueType::Address => SimulationPayloadFieldType::Address,
        GemEIP712ValueType::Timestamp => SimulationPayloadFieldType::Timestamp,
        GemEIP712ValueType::Text => SimulationPayloadFieldType::Text,
    };

    match canonical_payload_label(&field.name) {
        Some(CanonicalPayloadLabel::Kind(kind)) => MessagePayloadField::standard(kind, field.value.clone(), field_type, SimulationPayloadFieldDisplay::Secondary),
        Some(CanonicalPayloadLabel::Custom(label)) => MessagePayloadField::custom(label, field.value.clone(), field_type, SimulationPayloadFieldDisplay::Secondary),
        None => MessagePayloadField::custom(&field.name, field.value.clone(), field_type, SimulationPayloadFieldDisplay::Secondary),
    }
}

fn siwe_preview_fields(message: &SiweMessage) -> Vec<MessagePayloadField> {
    vec![
        MessagePayloadField::custom("domain", message.domain.clone(), SimulationPayloadFieldType::Text, SimulationPayloadFieldDisplay::Secondary),
        MessagePayloadField::custom(
            "address",
            message.address.clone(),
            SimulationPayloadFieldType::Address,
            SimulationPayloadFieldDisplay::Secondary,
        ),
        MessagePayloadField::custom("uri", message.uri.clone(), SimulationPayloadFieldType::Text, SimulationPayloadFieldDisplay::Secondary),
        MessagePayloadField::custom(
            "chainId",
            message.chain_id.to_string(),
            SimulationPayloadFieldType::Text,
            SimulationPayloadFieldDisplay::Secondary,
        ),
        MessagePayloadField::custom("nonce", message.nonce.clone(), SimulationPayloadFieldType::Text, SimulationPayloadFieldDisplay::Secondary),
        MessagePayloadField::custom(
            "issuedAt",
            message.issued_at.clone(),
            SimulationPayloadFieldType::Timestamp,
            SimulationPayloadFieldDisplay::Secondary,
        ),
        MessagePayloadField::custom(
            "version",
            message.version.clone(),
            SimulationPayloadFieldType::Text,
            SimulationPayloadFieldDisplay::Secondary,
        ),
    ]
}

fn payload_merge_key(field: &SimulationPayloadField) -> PayloadMergeKey {
    match field.kind {
        SimulationPayloadFieldKind::Custom => PayloadMergeKey::Label(canonical_merge_label(field.label.as_deref().unwrap_or_default()).into_owned()),
        _ => PayloadMergeKey::Kind(field.kind.clone()),
    }
}

fn canonical_payload_label(label: &str) -> Option<CanonicalPayloadLabel<'_>> {
    if label.chars().all(is_identifier_separator) {
        return None;
    }

    let kind = canonical_payload_kind(label);

    match kind {
        Some(kind) => Some(CanonicalPayloadLabel::Kind(kind)),
        None => Some(CanonicalPayloadLabel::Custom(canonical_merge_label(label))),
    }
}

fn canonical_payload_kind(label: &str) -> Option<SimulationPayloadFieldKind> {
    if identifier_eq(label, "contract") || identifier_eq(label, "verifyingContract") {
        return Some(SimulationPayloadFieldKind::Contract);
    }
    if identifier_eq(label, "method") || identifier_eq(label, "action") {
        return Some(SimulationPayloadFieldKind::Method);
    }
    if identifier_eq(label, "token") {
        return Some(SimulationPayloadFieldKind::Token);
    }
    if identifier_eq(label, "spender") {
        return Some(SimulationPayloadFieldKind::Spender);
    }
    if identifier_eq(label, "value") || identifier_eq(label, "amount") {
        return Some(SimulationPayloadFieldKind::Value);
    }
    if identifier_eq(label, "expiration") || identifier_eq(label, "expiry") {
        return Some(SimulationPayloadFieldKind::Expiration);
    }
    None
}

fn canonical_merge_label(label: &str) -> Cow<'_, str> {
    if identifier_eq(label, "domain") {
        return Cow::Borrowed("domain");
    }
    if identifier_eq(label, "address") {
        return Cow::Borrowed("address");
    }
    if identifier_eq(label, "uri") {
        return Cow::Borrowed("uri");
    }
    if identifier_eq(label, "chainId") {
        return Cow::Borrowed("chainId");
    }
    Cow::Borrowed(label)
}

fn identifier_eq(left: &str, right: &str) -> bool {
    let mut left = left.chars().filter(|character| !is_identifier_separator(*character));
    let mut right = right.chars().filter(|character| !is_identifier_separator(*character));

    loop {
        match (left.next(), right.next()) {
            (Some(left), Some(right)) if left.eq_ignore_ascii_case(&right) => {}
            (None, None) => return true,
            _ => return false,
        }
    }
}

fn is_identifier_separator(character: char) -> bool {
    character == ' ' || character == '_' || character == '-' || character == '.'
}

impl MessagePayloadField {
    fn standard(kind: SimulationPayloadFieldKind, value: String, field_type: SimulationPayloadFieldType, display: SimulationPayloadFieldDisplay) -> Self {
        Self {
            kind: Some(kind),
            label: None,
            value,
            field_type,
            display,
        }
    }

    fn custom(label: impl Into<String>, value: String, field_type: SimulationPayloadFieldType, display: SimulationPayloadFieldDisplay) -> Self {
        Self {
            kind: None,
            label: Some(label.into()),
            value,
            field_type,
            display,
        }
    }

    fn into_public_field(self) -> SimulationPayloadField {
        match self.kind {
            Some(kind) => SimulationPayloadField::standard(kind, self.value, self.field_type, self.display),
            None => SimulationPayloadField::custom(self.label.unwrap_or_default(), self.value, self.field_type, self.display),
        }
    }

    fn merge_key(&self) -> PayloadMergeKey {
        match self.kind.as_ref() {
            Some(kind) => PayloadMergeKey::Kind(kind.clone()),
            None => PayloadMergeKey::Label(canonical_merge_label(self.label.as_deref().unwrap_or_default()).into_owned()),
        }
    }
}

impl PayloadMergeKey {
    fn is_contract_action_key(&self) -> bool {
        match self {
            Self::Kind(SimulationPayloadFieldKind::Contract)
            | Self::Kind(SimulationPayloadFieldKind::Method)
            | Self::Kind(SimulationPayloadFieldKind::Token)
            | Self::Kind(SimulationPayloadFieldKind::Spender) => true,
            Self::Kind(SimulationPayloadFieldKind::Value)
            | Self::Kind(SimulationPayloadFieldKind::Expiration)
            | Self::Kind(SimulationPayloadFieldKind::Custom)
            | Self::Label(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MessagePayloadPreview;
    use crate::message::eip712::{GemEIP712Message, GemEIP712Section, GemEIP712Value, GemEIP712ValueType};
    use crate::message::sign_type::MessageType;
    use crate::siwe::SiweMessage;
    use gem_evm::EIP712Domain;
    use gem_solana::siws::SiwsMessage;
    use primitives::{SimulationPayloadField, SimulationPayloadFieldDisplay, SimulationPayloadFieldKind, SimulationPayloadFieldType};

    #[test]
    fn test_siws_preview_groups_identity_and_preserves_optional_fields() {
        let message = SiwsMessage::parse(include_str!("../../../crates/gem_solana/testdata/siws_complete.txt")).unwrap().unwrap();
        let preview = MessagePayloadPreview::from_siws(&message, vec![]);

        assert_eq!(preview.message_type, MessageType::Siws);

        assert_eq!(
            preview.primary,
            vec![
                SimulationPayloadField::custom("domain", "example.com", SimulationPayloadFieldType::Text, SimulationPayloadFieldDisplay::Primary),
                SimulationPayloadField::custom(
                    "address",
                    "AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9",
                    SimulationPayloadFieldType::Address,
                    SimulationPayloadFieldDisplay::Primary
                ),
            ]
        );
        assert_eq!(
            preview.secondary,
            [
                ("statement", "Sign in to the app.", SimulationPayloadFieldType::Text),
                ("uri", "https://example.com/login", SimulationPayloadFieldType::Text),
                ("version", "1", SimulationPayloadFieldType::Text),
                ("chainId", "mainnet", SimulationPayloadFieldType::Text),
                ("nonce", "8hK9pX32", SimulationPayloadFieldType::Text),
                ("issuedAt", "2026-09-01T12:00:00Z", SimulationPayloadFieldType::Timestamp),
                ("expirationTime", "2026-09-02T12:00:00Z", SimulationPayloadFieldType::Timestamp),
                ("notBefore", "2026-09-01T11:00:00Z", SimulationPayloadFieldType::Timestamp),
                ("requestId", "request-123", SimulationPayloadFieldType::Text),
                ("resources", "https://example.com/terms\nhttps://example.com/privacy", SimulationPayloadFieldType::Text),
            ]
            .into_iter()
            .map(|(label, value, field_type)| SimulationPayloadField::custom(label, value, field_type, SimulationPayloadFieldDisplay::Secondary))
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn siwe_preview_groups_domain_and_address_as_primary() {
        let preview = MessagePayloadPreview::from_siwe(
            &SiweMessage {
                domain: "login.xyz".into(),
                address: "0x123".into(),
                uri: "https://login.xyz".into(),
                chain_id: 1,
                nonce: "nonce".into(),
                version: "1".into(),
                issued_at: "2026-03-09T15:48:34.458Z".into(),
            },
            vec![],
        );

        assert_eq!(preview.message_type, MessageType::Siwe);
        assert_eq!(preview.primary.len(), 2);
        assert_eq!(preview.secondary.len(), 5);
    }

    #[test]
    fn eip712_preview_keeps_simulation_primary_payload() {
        let preview = GemEIP712Message {
            domain: EIP712Domain {
                name: Some("Permit2".into()),
                version: None,
                chain_id: Some(1),
                verifying_contract: Some("0xContract".into()),
                salts: None,
            },
            message: vec![GemEIP712Section {
                name: "PermitSingle".into(),
                values: vec![
                    GemEIP712Value {
                        name: "spender".into(),
                        value: "0xSpender".into(),
                        value_type: GemEIP712ValueType::Address,
                    },
                    GemEIP712Value {
                        name: "amount".into(),
                        value: "100".into(),
                        value_type: GemEIP712ValueType::Text,
                    },
                ],
            }],
        }
        .payload_preview(vec![
            SimulationPayloadField::standard(
                SimulationPayloadFieldKind::Contract,
                "0xContract",
                SimulationPayloadFieldType::Address,
                SimulationPayloadFieldDisplay::Primary,
            ),
            SimulationPayloadField::standard(
                SimulationPayloadFieldKind::Method,
                "Permit Single",
                SimulationPayloadFieldType::Text,
                SimulationPayloadFieldDisplay::Primary,
            ),
            SimulationPayloadField::standard(
                SimulationPayloadFieldKind::Spender,
                "0xSpender",
                SimulationPayloadFieldType::Address,
                SimulationPayloadFieldDisplay::Primary,
            ),
        ]);

        assert_eq!(preview.message_type, MessageType::Eip712);
        assert_eq!(preview.primary.len(), 3);
        assert_eq!(preview.secondary.len(), 2);
        assert_eq!(preview.secondary[0].label.as_deref(), Some("domain"));
        assert_eq!(preview.secondary[1].kind, SimulationPayloadFieldKind::Value);
    }

    #[test]
    fn preview_promotes_single_secondary_field() {
        let preview = GemEIP712Message {
            domain: EIP712Domain {
                name: None,
                version: None,
                chain_id: Some(1),
                verifying_contract: None,
                salts: None,
            },
            message: vec![GemEIP712Section {
                name: "Action".into(),
                values: vec![],
            }],
        }
        .payload_preview(vec![SimulationPayloadField::standard(
            SimulationPayloadFieldKind::Contract,
            "0xContract",
            SimulationPayloadFieldType::Address,
            SimulationPayloadFieldDisplay::Primary,
        )]);

        assert_eq!(preview.primary.len(), 2);
        assert!(preview.secondary.is_empty());
    }

    #[test]
    fn preview_promotes_secondary_fields_when_primary_is_empty() {
        let preview = MessagePayloadPreview::from_siwe(
            &SiweMessage {
                domain: "login.xyz".into(),
                address: "0x123".into(),
                uri: "https://login.xyz".into(),
                chain_id: 1,
                nonce: "nonce".into(),
                version: "1".into(),
                issued_at: "2026-03-09T15:48:34.458Z".into(),
            },
            vec![SimulationPayloadField::custom(
                "customField",
                "value",
                SimulationPayloadFieldType::Text,
                SimulationPayloadFieldDisplay::Secondary,
            )],
        );

        assert_eq!(preview.primary.len(), 8);
        assert!(preview.primary.iter().any(|field| field.label.as_deref() == Some("customField")));
        assert!(preview.secondary.is_empty());
    }
}
