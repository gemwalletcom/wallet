use primitives::{Chain, Wallet};

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemWalletImportType {
    MulticoinPhrase { words: Vec<String>, chains: Vec<Chain> },
    SinglePhrase { words: Vec<String>, chain: Chain },
    PrivateKey { value: String, chain: Chain },
    Address { address: String, chain: Chain },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemWalletImportKind {
    Phrase,
    PrivateKey,
    Address,
}

#[uniffi::export]
impl GemWalletImportKind {
    pub fn protects_input(&self) -> bool {
        !matches!(self, Self::Address)
    }

    pub fn supports_phrase_suggestions(&self) -> bool {
        matches!(self, Self::Phrase)
    }

    pub fn shows_view_only_warning(&self) -> bool {
        matches!(self, Self::Address)
    }
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
