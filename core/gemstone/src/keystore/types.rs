use std::fmt;

use gem_keystore::SecretKind;
use primitives::{Account, Chain, WalletId, WalletType};

pub type GemSecretKind = SecretKind;

#[uniffi::remote(Enum)]
pub enum GemSecretKind {
    Mnemonic,
    PrivateKey,
}

pub type GemWalletType = WalletType;

#[uniffi::remote(Enum)]
pub enum GemWalletType {
    Multicoin,
    Single,
    PrivateKey,
    View,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemKeystoreAccount {
    pub chain: Chain,
    pub address: String,
    pub derivation_path: String,
    pub public_key: Option<String>,
}

impl From<Account> for GemKeystoreAccount {
    fn from(account: Account) -> Self {
        Self {
            chain: account.chain,
            address: account.address,
            derivation_path: account.derivation_path,
            public_key: Some(account.extended_public_key.unwrap_or_default()),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemWalletImport {
    pub wallet_id: String,
    pub wallet_type: GemWalletType,
    pub accounts: Vec<GemKeystoreAccount>,
}

impl GemWalletImport {
    pub(super) fn new(wallet_id: WalletId, wallet_type: WalletType, accounts: Vec<Account>) -> Self {
        Self {
            wallet_id: wallet_id.to_string(),
            wallet_type,
            accounts: accounts.into_iter().map(GemKeystoreAccount::from).collect(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemStoredWallet {
    pub wallet_id: String,
    pub wallet_type: GemWalletType,
    pub keystore_id: String,
    pub accounts: Vec<GemKeystoreAccount>,
}

impl GemStoredWallet {
    pub(super) fn new(wallet_id: WalletId, wallet_type: WalletType, keystore_id: String, accounts: Vec<Account>) -> Self {
        Self {
            wallet_id: wallet_id.to_string(),
            wallet_type,
            keystore_id,
            accounts: accounts.into_iter().map(GemKeystoreAccount::from).collect(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemStoredSecretMigration {
    pub keystore_id: String,
    pub kind: GemSecretKind,
}

#[derive(Clone, uniffi::Enum)]
pub enum GemImportType {
    MulticoinPhrase { words: Vec<String>, chains: Vec<Chain> },
    SinglePhrase { words: Vec<String>, chain: Chain },
    PrivateKey { value: String, chain: Chain },
}

impl fmt::Debug for GemImportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GemImportType::MulticoinPhrase { words, chains } => f.debug_struct("MulticoinPhrase").field("word_count", &words.len()).field("chains", chains).finish(),
            GemImportType::SinglePhrase { words, chain } => f.debug_struct("SinglePhrase").field("word_count", &words.len()).field("chain", chain).finish(),
            GemImportType::PrivateKey { chain, .. } => f.debug_struct("PrivateKey").field("chain", chain).field("value", &"<redacted>").finish(),
        }
    }
}
