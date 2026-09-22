use std::fmt;

use crate::mnemonic::{apply_phrase_suggestion, phrase_suggestions};
use crate::services::localization::GemLocalizedText;
use primitives::{BlockExplorerLink, Chain, ChainAddress, NameRecord, Wallet, WalletSource};

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

#[derive(Clone, PartialEq, uniffi::Record)]
pub struct GemWalletImportSession {
    pub kind: GemWalletImportKind,
    pub text: String,
    pub cursor: Option<u32>,
    pub is_importing: bool,
}

impl fmt::Debug for GemWalletImportSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GemWalletImportSession")
            .field("kind", &self.kind)
            .field("cursor", &self.cursor)
            .field("is_importing", &self.is_importing)
            .finish_non_exhaustive()
    }
}

#[uniffi::export]
impl GemWalletImportSession {
    pub fn on_kind_changed(&self, kind: GemWalletImportKind) -> Self {
        Self {
            kind,
            text: String::new(),
            cursor: None,
            is_importing: false,
        }
    }

    pub fn on_input_changed(&self, text: String, cursor: Option<u32>) -> Self {
        Self { text, cursor, ..self.clone() }
    }

    pub fn on_suggestion_selected(&self, word: String) -> Self {
        let edit = apply_phrase_suggestion(&self.text, self.input_cursor(), &word);
        Self {
            text: edit.text,
            cursor: Some(edit.cursor),
            ..self.clone()
        }
    }

    pub fn on_importing(&self, is_importing: bool) -> Self {
        Self { is_importing, ..self.clone() }
    }

    pub fn suggestions(&self) -> Vec<String> {
        match self.kind {
            GemWalletImportKind::Phrase => phrase_suggestions(&self.text, self.input_cursor()),
            GemWalletImportKind::PrivateKey | GemWalletImportKind::Address => vec![],
        }
    }
}

impl GemWalletImportSession {
    fn input_cursor(&self) -> u32 {
        let end = self.text.encode_utf16().count() as u32;
        self.cursor.map_or(end, |cursor| cursor.min(end))
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
    New { wallet: Wallet, has_existing_wallets: bool },
    Existing { wallet: Wallet },
}

#[uniffi::export]
impl GemWalletImportResult {
    pub fn wallet(&self) -> Wallet {
        match self {
            Self::New { wallet, .. } | Self::Existing { wallet } => wallet.clone(),
        }
    }

    pub fn has_existing_wallets(&self) -> bool {
        match self {
            Self::New { has_existing_wallets, .. } => *has_existing_wallets,
            Self::Existing { .. } => true,
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
    pub address_explorer: Option<BlockExplorerLink>,
}

pub fn wallet_details(wallet: Wallet) -> GemWalletDetails {
    rules::details(&wallet)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemWalletSecretKind {
    Phrase,
    PrivateKey,
}

#[cfg(test)]
mod import_session_tests {
    use super::*;

    fn session(kind: GemWalletImportKind) -> GemWalletImportSession {
        GemWalletImportSession {
            kind,
            text: String::new(),
            cursor: None,
            is_importing: false,
        }
    }

    #[test]
    fn test_suggestions_follow_the_cursor_and_only_a_phrase_gets_them() {
        let typed = session(GemWalletImportKind::Phrase).on_input_changed("abandon woo zoo".into(), Some(11));

        assert_eq!(typed.suggestions(), vec!["wood", "wool"]);
        assert_eq!(typed.on_input_changed("woo".into(), None).suggestions(), vec!["wood", "wool"], "no cursor reads as the end");
        assert_eq!(typed.on_input_changed("woo".into(), Some(99)).suggestions(), vec!["wood", "wool"], "a stale cursor is clamped");
        assert!(session(GemWalletImportKind::PrivateKey).on_input_changed("woo".into(), None).suggestions().is_empty());
    }

    #[test]
    fn test_selecting_a_suggestion_replaces_the_word_and_moves_the_cursor() {
        let picked = session(GemWalletImportKind::Phrase).on_input_changed("abandon woo".into(), None).on_suggestion_selected("wood".into());

        assert_eq!((picked.text.as_str(), picked.cursor), ("abandon wood ", Some(13)));
    }

    #[test]
    fn test_changing_the_kind_clears_the_input() {
        let changed = session(GemWalletImportKind::Phrase)
            .on_input_changed("abandon".into(), Some(3))
            .on_importing(true)
            .on_kind_changed(GemWalletImportKind::Address);

        assert_eq!(changed, session(GemWalletImportKind::Address));
    }

    #[test]
    fn test_debug_never_prints_the_secret() {
        let debug = format!("{:?}", session(GemWalletImportKind::Phrase).on_input_changed("abandon ability".into(), None));

        assert!(!debug.contains("abandon"), "{debug}");
    }

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
