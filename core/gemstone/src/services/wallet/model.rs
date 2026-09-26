use std::fmt;

use crate::models::list::GemAddressRow;
use crate::services::localization::GemLocalizedText;
use primitives::{Chain, NameRecord, Wallet, WalletSource};

use super::rules;

#[derive(Clone, uniffi::Enum)]
pub enum GemWalletImportType {
    MulticoinPhrase { words: Vec<String>, chains: Vec<Chain> },
    SinglePhrase { words: Vec<String>, chain: Chain },
    PrivateKey { value: String, chain: Chain },
    Address { address: String, chain: Chain },
}

impl fmt::Debug for GemWalletImportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MulticoinPhrase { words, chains } => f.debug_struct("MulticoinPhrase").field("word_count", &words.len()).field("chains", chains).finish(),
            Self::SinglePhrase { words, chain } => f.debug_struct("SinglePhrase").field("word_count", &words.len()).field("chain", chain).finish(),
            Self::PrivateKey { chain, .. } => f.debug_struct("PrivateKey").field("chain", chain).finish_non_exhaustive(),
            Self::Address { address, chain } => f.debug_struct("Address").field("address", address).field("chain", chain).finish(),
        }
    }
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

    pub fn resolves_names(&self) -> bool {
        matches!(self, Self::Address)
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemWalletImportScreen {
    pub title: GemLocalizedText,
    pub kinds: Vec<GemWalletImportKind>,
    pub shows_kinds: bool,
}

#[derive(Clone, uniffi::Record)]
pub struct GemWalletImportRequest {
    pub kind: GemWalletImportKind,
    pub chain: Option<Chain>,
    pub input: String,
    pub name_record: Option<NameRecord>,
    pub default_name: String,
    pub source: WalletSource,
}

impl fmt::Debug for GemWalletImportRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GemWalletImportRequest")
            .field("kind", &self.kind)
            .field("chain", &self.chain)
            .field("default_name", &self.default_name)
            .field("source", &self.source)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemWalletImportResult {
    New { wallet: Wallet },
    Existing { wallet: Wallet },
}

#[uniffi::export]
impl GemWalletImportResult {
    pub fn wallet(&self) -> Wallet {
        match self {
            Self::New { wallet, .. } | Self::Existing { wallet } => wallet.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemWalletDeletion {
    WalletsRemaining,
    LastWalletDeleted,
}

#[derive(Clone, PartialEq, uniffi::Enum)]
pub enum GemWalletSecret {
    Words { words: Vec<String> },
    PrivateKey { key: String },
}

impl fmt::Debug for GemWalletSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Words { words } => f.debug_struct("Words").field("word_count", &words.len()).finish(),
            Self::PrivateKey { .. } => f.debug_struct("PrivateKey").finish_non_exhaustive(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemSecretPhraseRow {
    Pair { left: u32, right: u32 },
    Single { index: u32 },
}

#[uniffi::export]
pub fn secret_phrase_rows(word_count: u32) -> Vec<GemSecretPhraseRow> {
    rules::secret_phrase_rows(word_count)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSecretWarning {
    DoNotShare,
    SaveSafely,
}

/// What the secret screen shows around the secret, never the secret itself: the words stay in the app and
/// fill `rows` by index, so this record is built from the kind and the word count alone.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSecretScreen {
    pub title: GemLocalizedText,
    pub warning: GemSecretWarning,
    pub rows: Vec<GemSecretPhraseRow>,
}

#[uniffi::export]
pub fn secret_screen(kind: GemWalletSecretKind, word_count: u32, is_new: bool) -> GemSecretScreen {
    rules::secret_screen(kind, word_count, is_new)
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemWalletSubtitle {
    Multicoin,
    Address { value: String },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemWalletPlaceholder {
    Multicoin,
    Chain { chain: Chain },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemWalletRow {
    pub id: String,
    pub name: String,
    pub subtitle: GemWalletSubtitle,
    pub placeholder: GemWalletPlaceholder,
    pub shows_watch_badge: bool,
    pub is_pinned: bool,
    pub is_current: bool,
    pub has_avatar: bool,
    pub image_url: Option<String>,
}

#[uniffi::export]
pub fn wallet_row(wallet: Wallet) -> GemWalletRow {
    rules::row(&wallet)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemWalletSectionKind {
    Pinned,
    Wallets,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemWalletSection {
    pub kind: GemWalletSectionKind,
    pub rows: Vec<GemWalletRow>,
}

#[uniffi::export]
pub fn wallet_sections(wallets: Vec<Wallet>, current_wallet_id: Option<String>) -> Vec<GemWalletSection> {
    rules::sections(wallets, current_wallet_id.as_deref())
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemWalletDetails {
    pub row: GemWalletRow,
    pub secret_kind: Option<GemWalletSecretKind>,
    pub show_secret: Option<GemLocalizedText>,
    pub address: Option<GemAddressRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemWalletSecretKind {
    Phrase,
    PrivateKey,
}

#[cfg(test)]
mod secret_debug_tests {
    use super::*;

    #[test]
    fn test_debug_never_prints_an_import_or_exported_secret() {
        let words = vec!["abandon".to_string(), "ability".to_string()];
        let printed = format!(
            "{:?} {:?} {:?} {:?} {:?}",
            GemWalletImportType::MulticoinPhrase {
                words: words.clone(),
                chains: vec![Chain::Ethereum]
            },
            GemWalletImportType::SinglePhrase { words: words.clone(), chain: Chain::Bitcoin },
            GemWalletImportType::PrivateKey {
                value: "0xsecretkey".into(),
                chain: Chain::Ethereum
            },
            GemWalletSecret::Words { words },
            GemWalletSecret::PrivateKey { key: "0xsecretkey".into() },
        );

        assert!(!printed.contains("abandon") && !printed.contains("ability") && !printed.contains("secretkey"), "{printed}");
    }
}
