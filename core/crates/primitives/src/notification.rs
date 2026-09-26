use crate::{CoreListItem, WalletId};
use chrono::{DateTime, Utc};
use model_derive::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[model(swift = "Sendable, Equatable")]
#[serde(rename_all = "camelCase")]
pub struct InAppNotification {
    pub wallet_id: WalletId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub item: CoreListItem,
}
