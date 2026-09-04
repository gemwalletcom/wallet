use primitives::{Chain, Wallet};

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemWalletImportType {
    MulticoinPhrase { words: Vec<String>, chains: Vec<Chain> },
    SinglePhrase { words: Vec<String>, chain: Chain },
    PrivateKey { value: String, chain: Chain },
    Address { address: String, chain: Chain },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemWalletDefaultName {
    pub name: String,
    pub has_existing_wallets: bool,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemWalletImportResult {
    New { wallet: Wallet },
    Existing { wallet: Wallet },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemWalletDeletion {
    WalletsRemaining,
    LastWalletDeleted,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemWalletSecret {
    Words { words: Vec<String> },
    PrivateKey { key: String },
}
