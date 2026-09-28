use primitives::Latency;

use super::rules;
use crate::formatted_number::GemFormattedNumber;
use crate::gateway::GatewayError;
use crate::models::placeholder::text_or_placeholder;
use crate::services::localization::GemLocalizedText;
use crate::services::service_status::GemLatencyStatus;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemNodeStatusState {
    Loading,
    Error,
    Result { latest_block_number: u64, latency: Latency },
}

impl GemNodeStatusState {
    pub fn latency_status(&self) -> GemLatencyStatus {
        rules::latency_status(self)
    }

    pub fn subtitle(&self) -> GemNodeSubtitle {
        GemNodeSubtitle::LatestBlock {
            value: self.latest_block().map(GemFormattedNumber::count),
        }
    }

    pub fn latest_block(&self) -> Option<u64> {
        match self {
            Self::Result { latest_block_number, .. } => Some(*latest_block_number),
            Self::Loading | Self::Error => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemNodeCheck {
    pub url: String,
    pub chain_id: Option<String>,
    pub latest_block_number: u64,
    pub is_in_sync: bool,
    pub latency: Latency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemNodeSyncState {
    InSync,
    OutOfSync,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemNodeCheckRow {
    ChainId { value: String },
    InSync { state: GemNodeSyncState },
    LatestBlock { value: GemFormattedNumber },
    Latency { milliseconds: u32 },
}

impl GemNodeCheck {
    pub fn rows(&self) -> Vec<GemNodeCheckRow> {
        vec![
            GemNodeCheckRow::ChainId {
                value: text_or_placeholder(self.chain_id.as_deref()),
            },
            GemNodeCheckRow::InSync {
                state: match self.is_in_sync {
                    true => GemNodeSyncState::InSync,
                    false => GemNodeSyncState::OutOfSync,
                },
            },
            GemNodeCheckRow::LatestBlock {
                value: GemFormattedNumber::count(self.latest_block_number),
            },
            GemNodeCheckRow::Latency { milliseconds: self.latency.value as u32 },
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemExplorerRow {
    pub name: String,
    pub is_selected: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemNodeSubtitle {
    LatestBlock { value: Option<GemFormattedNumber> },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemNodeRow {
    pub node: GemNodeSelection,
    pub title: GemNodeRowTitle,
    pub subtitle: GemNodeSubtitle,
    pub latency_status: GemLatencyStatus,
    pub can_delete: bool,
    pub delete_prompt: GemLocalizedText,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemChainSettingsSection {
    Nodes { rows: Vec<GemNodeRow> },
    Explorers { rows: Vec<GemExplorerRow> },
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemNodeRowTitle {
    Host { host: String },
    GemNode { flag: String },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemNodeSelection {
    pub url: String,
    pub host: String,
    pub is_selected: bool,
    pub gem_node_flag: Option<String>,
}

#[uniffi::export]
impl GemNodeSubtitle {
    pub fn text(&self, latest_block_label: String, latest_block_value: Option<String>) -> String {
        match self {
            Self::LatestBlock { .. } => format!("{latest_block_label}: {}", text_or_placeholder(latest_block_value.as_deref())),
        }
    }
}

#[uniffi::export]
impl GemNodeRowTitle {
    pub fn text(&self, gem_node_label: String) -> String {
        match self {
            Self::Host { host } => host.clone(),
            Self::GemNode { flag } => format!("{gem_node_label} {flag}"),
        }
    }
}

impl GemNodeSelection {
    pub fn title(&self) -> GemNodeRowTitle {
        match &self.gem_node_flag {
            Some(flag) => GemNodeRowTitle::GemNode { flag: flag.clone() },
            None => GemNodeRowTitle::Host { host: self.host.clone() },
        }
    }
}

#[derive(Debug, Clone, uniffi::Error)]
pub enum GemAddNodeError {
    InvalidUrl,
    InvalidNetworkId,
    Gateway(GatewayError),
}

impl std::fmt::Display for GemAddNodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUrl => write!(f, "invalid node url"),
            Self::InvalidNetworkId => write!(f, "node answered for a different network"),
            Self::Gateway(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for GemAddNodeError {}

impl From<GatewayError> for GemAddNodeError {
    fn from(error: GatewayError) -> Self {
        match error {
            GatewayError::NetworkIdMismatch { .. } => Self::InvalidNetworkId,
            error => Self::Gateway(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_node_row_subtitle_names_the_latest_block_and_admits_when_it_has_none() {
        let result = GemNodeStatusState::mock_result(21_000_000);

        assert_eq!(
            result.subtitle(),
            GemNodeSubtitle::LatestBlock {
                value: Some(GemFormattedNumber::count(21_000_000))
            }
        );
        assert_eq!(GemNodeStatusState::Loading.subtitle(), GemNodeSubtitle::LatestBlock { value: None });
        assert_eq!(
            GemNodeStatusState::Error.subtitle(),
            GemNodeSubtitle::LatestBlock { value: None },
            "a node that failed still shows the block row, with nothing in it"
        );
    }

    #[test]
    fn test_a_node_subtitle_prints_the_label_before_the_block() {
        let subtitle = GemNodeSubtitle::LatestBlock { value: None };

        assert_eq!(subtitle.text("Latest block".to_string(), Some("21,000,000".to_string())), "Latest block: 21,000,000");
        assert_eq!(subtitle.text("Latest block".to_string(), None), "Latest block: -");
    }

    #[test]
    fn test_a_checked_node_shows_the_same_four_rows_whatever_it_answered() {
        let check = GemNodeCheck {
            url: "https://node".to_string(),
            chain_id: None,
            latest_block_number: 21_000_000,
            is_in_sync: false,
            latency: Latency::from_milliseconds(120),
        };

        assert_eq!(
            check.rows(),
            vec![
                GemNodeCheckRow::ChainId { value: "-".to_string() },
                GemNodeCheckRow::InSync { state: GemNodeSyncState::OutOfSync },
                GemNodeCheckRow::LatestBlock {
                    value: GemFormattedNumber::count(21_000_000)
                },
                GemNodeCheckRow::Latency { milliseconds: 120 },
            ]
        );

        let synced = GemNodeCheck {
            chain_id: Some("1".to_string()),
            is_in_sync: true,
            ..check
        };

        assert_eq!(
            synced.rows(),
            vec![
                GemNodeCheckRow::ChainId { value: "1".to_string() },
                GemNodeCheckRow::InSync { state: GemNodeSyncState::InSync },
                GemNodeCheckRow::LatestBlock {
                    value: GemFormattedNumber::count(21_000_000)
                },
                GemNodeCheckRow::Latency { milliseconds: 120 },
            ]
        );
    }
}
