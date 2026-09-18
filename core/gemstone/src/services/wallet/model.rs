use crate::services::localization::GemLocalizedText;
use primitives::{Chain, ChainAddress, Wallet};

use super::rules;

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

    pub fn resolves_names(&self) -> bool {
        matches!(self, Self::Address)
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemWalletDefaultName {
    pub text: GemLocalizedText,
    pub has_existing_wallets: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemWalletImportScreen {
    pub title: GemLocalizedText,
    pub kinds: Vec<GemWalletImportKind>,
    pub shows_kinds: bool,
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

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemSecretPhraseRow {
    Pair { left: u32, right: u32 },
    Single { index: u32 },
}

#[uniffi::export]
pub fn secret_phrase_rows(word_count: u32) -> Vec<GemSecretPhraseRow> {
    rules::secret_phrase_rows(word_count)
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
    pub has_avatar: bool,
    pub image_url: Option<String>,
}

#[uniffi::export]
pub fn wallet_row(wallet: Wallet) -> GemWalletRow {
    rules::row(&wallet)
}

#[uniffi::export]
pub fn wallet_rows(wallets: Vec<Wallet>) -> Vec<GemWalletRow> {
    rules::rows(&wallets)
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemWalletDetails {
    pub row: GemWalletRow,
    pub secret_kind: Option<GemWalletSecretKind>,
    pub address: Option<ChainAddress>,
}

#[uniffi::export]
pub fn wallet_details(wallet: Wallet) -> GemWalletDetails {
    rules::details(&wallet)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemWalletSecretKind {
    Phrase,
    PrivateKey,
}
