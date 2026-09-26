use std::fmt;

use primitives::Chain;

use crate::address_formatter::{GemAddressFormatStyle, format_address};

const SECRET_CLIPBOARD_SECONDS: u32 = 60;

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemCopyKind {
    Address { chain: Chain },
    Plain,
    SecretPhrase,
    PrivateKey,
}

#[uniffi::export]
impl GemCopyKind {
    pub fn is_sensitive(&self) -> bool {
        match self {
            Self::Address { .. } | Self::Plain => false,
            Self::SecretPhrase | Self::PrivateKey => true,
        }
    }

    pub fn clipboard_expiry_seconds(&self) -> Option<u32> {
        self.is_sensitive().then_some(SECRET_CLIPBOARD_SECONDS)
    }
}

#[derive(Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemCopy {
    pub kind: GemCopyKind,
    pub value: String,
    pub display: String,
}

impl fmt::Debug for GemCopy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.kind.is_sensitive() {
            f.debug_struct("GemCopy").field("kind", &self.kind).finish_non_exhaustive()
        } else {
            f.debug_struct("GemCopy").field("kind", &self.kind).field("value", &self.value).field("display", &self.display).finish()
        }
    }
}

#[uniffi::export]
pub fn address_copy(chain: Chain, address: String) -> GemCopy {
    GemCopy {
        kind: GemCopyKind::Address { chain },
        display: format_address(&address, Some(chain), GemAddressFormatStyle::Short),
        value: address,
    }
}

#[uniffi::export]
pub fn secret_phrase_copy(words: Vec<String>) -> GemCopy {
    GemCopy {
        kind: GemCopyKind::SecretPhrase,
        value: words.join(" "),
        display: String::new(),
    }
}

#[uniffi::export]
pub fn private_key_copy(key: String) -> GemCopy {
    GemCopy {
        kind: GemCopyKind::PrivateKey,
        value: key,
        display: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_only_secrets_are_sensitive_and_an_address_copy_shortens_its_display() {
        let address = address_copy(Chain::Ethereum, "0x1234567890abcdef1234567890abcdef12345678".to_string());

        assert!(!address.kind.is_sensitive());
        assert_eq!(address.value, "0x1234567890abcdef1234567890abcdef12345678");
        assert_ne!(address.display, address.value);
        assert!(secret_phrase_copy(vec!["a".to_string(), "b".to_string()]).kind.is_sensitive());
        assert_eq!(secret_phrase_copy(vec!["a".to_string(), "b".to_string()]).value, "a b");
        assert!(private_key_copy("key".to_string()).kind.is_sensitive());
        assert!(private_key_copy("key".to_string()).display.is_empty());
    }

    #[test]
    fn test_a_copied_secret_leaves_the_clipboard_after_a_minute_and_an_address_stays() {
        assert_eq!(GemCopyKind::SecretPhrase.clipboard_expiry_seconds(), Some(60));
        assert_eq!(GemCopyKind::PrivateKey.clipboard_expiry_seconds(), Some(60));
        assert_eq!(GemCopyKind::Address { chain: Chain::Ethereum }.clipboard_expiry_seconds(), None);
        assert_eq!(GemCopyKind::Plain.clipboard_expiry_seconds(), None);
    }

    #[test]
    fn test_debug_prints_an_address_but_never_a_secret() {
        let printed = format!("{:?} {:?}", secret_phrase_copy(vec!["abandon".to_string(), "ability".to_string()]), private_key_copy("0xsecretkey".to_string()));

        assert!(!printed.contains("abandon") && !printed.contains("secretkey"), "{printed}");
        assert!(format!("{:?}", address_copy(Chain::Ethereum, "0xabc".to_string())).contains("0xabc"));
    }
}
