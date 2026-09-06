#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Wallet {
    pub id: WalletId,
    pub accounts: Vec<Account>,
    pub image_url: Option<String>,
}

impl Wallet {
    pub fn account(&self, chain: Chain) -> Option<&Account> {
        self.accounts.iter().find(|account| account.chain == chain)
    }
}
