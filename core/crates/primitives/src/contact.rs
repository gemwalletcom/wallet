use chrono::{DateTime, Utc};
use model_derive::Model;
use serde::{Deserialize, Serialize};

use crate::Chain;

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[model(swift = "Equatable, Hashable, Sendable, Identifiable")]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[model(swift = "Equatable, Hashable, Sendable, Identifiable")]
#[serde(rename_all = "camelCase")]
pub struct ContactAddress {
    pub id: String,
    pub contact_id: String,
    pub address: String,
    pub chain: Chain,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[model(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ContactData {
    pub contact: Contact,
    pub addresses: Vec<ContactAddress>,
}
