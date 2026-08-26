use primitives::{Chain, Wallet, WalletSource};

pub type GemWalletSource = WalletSource;

#[uniffi::remote(Enum)]
pub enum GemWalletSource {
    Create,
    Import,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemWalletImportType {
    MulticoinPhrase { words: Vec<String>, chains: Vec<Chain> },
    SinglePhrase { words: Vec<String>, chain: Chain },
    PrivateKey { value: String, chain: Chain },
    Address { address: String, chain: Chain },
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemWalletImportResult {
    New { wallet: Wallet },
    Existing { wallet: Wallet },
}
