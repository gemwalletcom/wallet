use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    #[serde(deserialize_with = "serde_serializers::deserialize_u64_from_str")]
    pub account_number: u64,
    #[serde(deserialize_with = "serde_serializers::deserialize_u64_from_str")]
    pub sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountResponse<T> {
    pub account: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectiveAccount {
    pub base_account: Account,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Pagination {
    pub next_key: Option<String>,
}

impl Pagination {
    pub(crate) fn key(pagination: &Option<Self>) -> Option<String> {
        pagination.as_ref()?.next_key.clone().filter(|key| !key.is_empty())
    }
}

pub trait Page {
    type Item;

    fn into_items(self) -> Vec<Self::Item>;
    fn next_page_key(&self) -> Option<String>;
}

impl Page for Balances {
    type Item = Balance;

    fn into_items(self) -> Vec<Balance> {
        self.balances
    }

    fn next_page_key(&self) -> Option<String> {
        Pagination::key(&self.pagination)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balances {
    pub balances: Vec<Balance>,
    #[serde(default)]
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub denom: String,
    pub amount: String,
}
