use primitives::Chain;

use crate::address_formatter::{GemAddressFormatStyle, format_address};

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemCopyKind {
    Address { chain: Chain },
    SecretPhrase,
    PrivateKey,
}

#[uniffi::export]
impl GemCopyKind {
    pub fn is_sensitive(&self) -> bool {
        match self {
            Self::Address { .. } => false,
            Self::SecretPhrase | Self::PrivateKey => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemCopy {
    pub kind: GemCopyKind,
    pub value: String,
    pub display: String,
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
}
