use primitives::Latency;

use super::rules;
use crate::gateway::GatewayError;
use crate::service_status::GemLatencyStatus;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemNodeStatusState {
    Loading,
    Error,
    Result { latest_block_number: u64, latency: Latency },
}

#[uniffi::export]
impl GemNodeStatusState {
    pub fn latency_status(&self) -> GemLatencyStatus {
        rules::latency_status(self)
    }

    pub fn latest_block(&self) -> Option<u64> {
        match self {
            Self::Result { latest_block_number, .. } => Some(*latest_block_number),
            Self::Loading | Self::Error => None,
        }
    }

    pub fn subtitle(&self) -> GemNodeSubtitle {
        GemNodeSubtitle::LatestBlock {
            value: rules::block_number_text(self.latest_block()),
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
    LatestBlock { value: String },
    Latency { milliseconds: u32 },
}

#[uniffi::export]
impl GemNodeCheck {
    pub fn rows(&self) -> Vec<GemNodeCheckRow> {
        vec![
            GemNodeCheckRow::ChainId {
                value: rules::text_or_placeholder(self.chain_id.as_deref()),
            },
            GemNodeCheckRow::InSync {
                state: match self.is_in_sync {
                    true => GemNodeSyncState::InSync,
                    false => GemNodeSyncState::OutOfSync,
                },
            },
            GemNodeCheckRow::LatestBlock {
                value: rules::block_number_text(Some(self.latest_block_number)),
            },
            GemNodeCheckRow::Latency {
                milliseconds: self.latency.value as u32,
            },
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemChainSettingsSection {
    Nodes,
    Explorer,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemExplorerRow {
    pub name: String,
    pub is_selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemNodeSubtitle {
    LatestBlock { value: String },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemNodeRow {
    pub node: GemNodeSelection,
    pub title: GemNodeRowTitle,
    pub subtitle: GemNodeSubtitle,
    pub latency_status: GemLatencyStatus,
    pub can_delete: bool,
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
        let result = GemNodeStatusState::Result {
            latest_block_number: 21_000_000,
            latency: Latency::from_milliseconds(120),
        };

        assert_eq!(result.subtitle(), GemNodeSubtitle::LatestBlock { value: "21,000,000".to_string() });
        assert_eq!(GemNodeStatusState::Loading.subtitle(), GemNodeSubtitle::LatestBlock { value: "-".to_string() });
        assert_eq!(
            GemNodeStatusState::Error.subtitle(),
            GemNodeSubtitle::LatestBlock { value: "-".to_string() },
            "a node that failed still shows the block row, with nothing in it"
        );
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
                GemNodeCheckRow::InSync {
                    state: GemNodeSyncState::OutOfSync
                },
                GemNodeCheckRow::LatestBlock { value: "21,000,000".to_string() },
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
                GemNodeCheckRow::LatestBlock { value: "21,000,000".to_string() },
                GemNodeCheckRow::Latency { milliseconds: 120 },
            ]
        );
    }
}
