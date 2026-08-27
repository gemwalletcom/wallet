use crate::{Account, WalletId, WalletType};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Default, Serialize, Deserialize, EnumString, AsRefStr, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum WalletSource {
    Create,
    #[default]
    Import,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct Wallet {
    pub id: WalletId,
    pub external_id: Option<String>,
    pub name: String,
    pub index: i32,
    #[serde(rename = "type")]
    pub wallet_type: WalletType,
    pub accounts: Vec<Account>,
    pub is_pinned: bool,
    pub image_url: Option<String>,
    pub source: WalletSource,
}

impl Wallet {
    pub fn account(&self, chain: crate::Chain) -> Option<&crate::Account> {
        self.accounts.iter().find(|account| account.chain == chain)
    }

    pub fn address_chains(&self) -> Vec<crate::AddressChains> {
        let mut chains_by_address: std::collections::BTreeMap<&str, std::collections::BTreeSet<crate::Chain>> = std::collections::BTreeMap::new();
        for account in &self.accounts {
            chains_by_address.entry(account.address.as_str()).or_default().insert(account.chain);
        }
        chains_by_address
            .into_iter()
            .map(|(address, chains)| crate::AddressChains::new(address.to_string(), chains.into_iter().collect()))
            .collect()
    }
}
