use primitives::Chain;

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum MessageType {
    Text,
    Eip712,
    Siwe,
    Siws,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum SignDigestType {
    Eip191,
    Eip712,
    Base58,
    SuiPersonal,
    Siwe,
    TonPersonal,
    TronPersonal,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SignMessage {
    pub chain: Chain,
    pub sign_type: SignDigestType,
    pub data: Vec<u8>,
}

impl MessageType {
    pub fn title(&self) -> crate::services::localization::GemLocalizedText {
        use crate::services::localization::GemLocalizedText;
        match self {
            Self::Siwe => GemLocalizedText::SignInWith { chain: primitives::Chain::Ethereum },
            Self::Siws => GemLocalizedText::SignInWith { chain: primitives::Chain::Solana },
            Self::Text | Self::Eip712 => GemLocalizedText::ReviewRequest,
        }
    }
}

#[cfg(test)]
mod title_tests {
    use super::*;
    use crate::services::localization::GemLocalizedText;
    use primitives::Chain;

    #[test]
    fn test_a_sign_in_names_its_chain_and_anything_else_is_a_review() {
        assert_eq!(MessageType::Siwe.title(), GemLocalizedText::SignInWith { chain: Chain::Ethereum });
        assert_eq!(MessageType::Siws.title(), GemLocalizedText::SignInWith { chain: Chain::Solana });
        assert_eq!(MessageType::Text.title(), GemLocalizedText::ReviewRequest);
        assert_eq!(MessageType::Eip712.title(), GemLocalizedText::ReviewRequest);
    }
}
