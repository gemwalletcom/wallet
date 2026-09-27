use crate::GemstoneError;
use crate::alien::AlienError;
use crate::gateway::GatewayError;
use crate::payment::GemPaymentError;
use crate::services::error::GemServiceError;
use crate::services::node::model::GemAddNodeError;
use crate::services::wallet::error::GemWalletImportError;
use crate::services::wallet_connect::error::GemWalletConnectError;
use primitives::PaymentStatus;

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemErrorText {
    Cancelled,
    NetworkOffline,
    NetworkMessage { text: String },
    NetworkStatus { status: u32 },
    InvalidNetworkId,
    InvalidUrl,
    NotSupported,
    UnsupportedChain,
    MaliciousOrigin,
    NoSupportedWallets,
    Payment { status: PaymentStatus },
    InvalidSecretPhrase,
    InvalidSecretPhraseWords { words: Vec<String> },
    InvalidPrivateKey,
    InvalidAddress,
    NoAccountForChain,
    AuthenticationUnavailable,
    AuthenticationLockedOut,
    AuthenticationFailed,
    ConnectionExpired,
    ConnectionNotFound,
    RelayUnavailable,
    Unknown,
    Message { text: String },
}

impl GemErrorText {
    pub fn message(text: String) -> Self {
        match text.trim().is_empty() {
            true => Self::Unknown,
            false => Self::Message { text },
        }
    }

    fn network_message(text: String) -> Self {
        match text.trim().is_empty() {
            true => Self::Unknown,
            false => Self::NetworkMessage { text },
        }
    }
}

#[uniffi::export]
impl GemServiceError {
    pub fn text(&self) -> GemErrorText {
        match self {
            Self::Api { msg } | Self::Gateway { msg } | Self::Platform { msg } | Self::InvalidInput { msg } | Self::Unsupported { msg } => GemErrorText::message(msg.clone()),
            Self::Store { .. } | Self::Core { .. } | Self::NotFound { .. } => GemErrorText::Unknown,
            Self::NoAccountForChain { .. } => GemErrorText::NoAccountForChain,
            Self::Offline => GemErrorText::NetworkOffline,
            Self::WalletImport { error } => error.text(),
            Self::Cancelled => GemErrorText::Cancelled,
        }
    }
}

#[uniffi::export]
impl GemWalletImportError {
    pub fn text(&self) -> GemErrorText {
        match self {
            Self::InvalidSecretPhrase => GemErrorText::InvalidSecretPhrase,
            Self::InvalidSecretPhraseWords { words } => GemErrorText::InvalidSecretPhraseWords { words: words.clone() },
            Self::InvalidPrivateKey => GemErrorText::InvalidPrivateKey,
            Self::InvalidAddress => GemErrorText::InvalidAddress,
            Self::MissingChain => GemErrorText::Unknown,
        }
    }
}

#[uniffi::export]
impl GemstoneError {
    pub fn text(&self) -> GemErrorText {
        match self {
            Self::AnyError { msg } | Self::SignerError { msg, .. } => GemErrorText::message(msg.clone()),
            Self::Cancelled => GemErrorText::Cancelled,
        }
    }
}

#[uniffi::export]
impl GatewayError {
    pub fn text(&self) -> GemErrorText {
        match self {
            Self::Offline => GemErrorText::NetworkOffline,
            Self::NetworkError { msg } | Self::PlatformError { msg } => GemErrorText::message(msg.clone()),
            Self::NetworkIdMismatch { .. } => GemErrorText::InvalidNetworkId,
        }
    }
}

#[uniffi::export]
impl GemAddNodeError {
    pub fn text(&self) -> GemErrorText {
        match self {
            Self::InvalidUrl => GemErrorText::InvalidUrl,
            Self::InvalidNetworkId => GemErrorText::InvalidNetworkId,
            Self::Gateway(error) => error.text(),
        }
    }
}

#[uniffi::export]
impl GemWalletConnectError {
    pub fn text(&self) -> GemErrorText {
        match self {
            Self::UnsupportedChains => GemErrorText::UnsupportedChain,
            Self::InvalidOrigin => GemErrorText::MaliciousOrigin,
            Self::UnsupportedWallets => GemErrorText::NoSupportedWallets,
            Self::Service { msg } => GemErrorText::message(msg.clone()),
        }
    }
}

#[uniffi::export]
pub fn wallet_connect_error_text(message: String) -> GemErrorText {
    let text = message.to_lowercase();
    let mentions = |phrases: &[&str]| phrases.iter().any(|phrase| text.contains(phrase));
    if mentions(&["uri has expired", "uri expired", "pairing expired", "proposal expired"]) {
        return GemErrorText::ConnectionExpired;
    }
    if mentions(&["matching the topic", "sequence for given topic", "no matching key"]) {
        return GemErrorText::ConnectionNotFound;
    }
    if mentions(&["web socket", "websocket", "relay request timeout", "internet connection", "connection closed"]) {
        return GemErrorText::RelayUnavailable;
    }
    GemErrorText::message(message)
}

#[uniffi::export]
pub fn alien_error_text(error: AlienError) -> GemErrorText {
    match error {
        AlienError::RequestError { msg } | AlienError::ResponseError { msg } => GemErrorText::network_message(msg),
        AlienError::Http { status, .. } => GemErrorText::NetworkStatus { status: status as u32 },
        AlienError::Offline => GemErrorText::NetworkOffline,
    }
}

#[uniffi::export]
pub fn payment_error_text(error: GemPaymentError) -> GemErrorText {
    match error {
        GemPaymentError::NoPaymentOptions => GemErrorText::NotSupported,
        GemPaymentError::Status { status } => GemErrorText::Payment { status },
        GemPaymentError::InvalidRequest { reason } | GemPaymentError::Network { reason } => GemErrorText::message(reason),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GemApiError;

    #[test]
    fn test_a_blank_message_reads_as_unknown() {
        assert_eq!(GemServiceError::Api { msg: " ".into() }.text(), GemErrorText::Unknown);
        assert_eq!(alien_error_text(AlienError::RequestError { msg: String::new() }), GemErrorText::Unknown);
        assert_eq!(payment_error_text(GemPaymentError::Network { reason: String::new() }), GemErrorText::Unknown);
    }

    #[test]
    fn test_a_missing_account_names_the_chain() {
        let error = GemServiceError::NoAccountForChain { chain: primitives::Chain::Ethereum };

        assert_eq!(error.text(), GemErrorText::NoAccountForChain);
        assert_eq!(error.to_string(), "wallet has no ethereum account");
    }

    #[test]
    fn test_a_transport_message_is_named_apart_from_a_service_message() {
        assert_eq!(
            alien_error_text(AlienError::RequestError { msg: "timeout".into() }),
            GemErrorText::NetworkMessage { text: "timeout".into() },
            "a transport failure reads as a network error on both apps"
        );
        assert_eq!(alien_error_text(AlienError::Offline), GemErrorText::NetworkOffline);
        assert_eq!(GemServiceError::from(GatewayError::Offline).text(), GemErrorText::NetworkOffline);
        assert_eq!(GemServiceError::from(GemApiError::Network { msg: AlienError::Offline.to_string() }).text(), GemErrorText::NetworkOffline);
        assert_ne!(GemServiceError::from(GatewayError::NetworkError { msg: "reset".to_string() }).text(), GemErrorText::NetworkOffline);
        assert_eq!(alien_error_text(AlienError::Http { status: 503, len: 0 }), GemErrorText::NetworkStatus { status: 503 });
        assert_eq!(
            GatewayError::NetworkError { msg: "reverted".into() }.text(),
            GemErrorText::Message { text: "reverted".into() },
            "a message the gateway already phrased is shown as it is"
        );
    }

    #[test]
    fn test_an_import_error_reaches_the_apps_as_its_own_text() {
        let words = GemWalletImportError::InvalidSecretPhraseWords { words: vec!["abandom".to_string()] };

        assert_eq!(GemServiceError::from(words.clone()).text(), GemErrorText::InvalidSecretPhraseWords { words: vec!["abandom".to_string()] });
        assert_eq!(GemServiceError::from(GemWalletImportError::InvalidPrivateKey).text(), GemErrorText::InvalidPrivateKey);
        assert!(!GemServiceError::from(words).to_string().contains("abandom"), "the log text never repeats a phrase word");
    }

    #[test]
    fn test_a_carried_message_stays_the_message_and_a_decision_becomes_a_key() {
        assert_eq!(GemServiceError::Api { msg: "boom".into() }.text(), GemErrorText::Message { text: "boom".into() });
        assert_eq!(GemServiceError::Store { msg: "database is locked".into() }.text(), GemErrorText::Unknown, "storage text is internal");
        assert_eq!(GemServiceError::NotFound { msg: "wallet 7 not found".into() }.text(), GemErrorText::Unknown);
        assert_eq!(GemServiceError::Cancelled.text(), GemErrorText::Cancelled);
        assert_eq!(GatewayError::Offline.text(), GemErrorText::NetworkOffline);
        assert_eq!(
            GatewayError::NetworkIdMismatch {
                chain: "ethereum".into(),
                network_id: "1".into()
            }
            .text(),
            GemErrorText::InvalidNetworkId
        );
    }

    #[test]
    fn test_an_added_node_reports_the_gateway_answer_it_wraps() {
        assert_eq!(GemAddNodeError::InvalidUrl.text(), GemErrorText::InvalidUrl);
        assert_eq!(GemAddNodeError::Gateway(GatewayError::Offline).text(), GemErrorText::NetworkOffline);
        assert_eq!(alien_error_text(AlienError::Http { status: 503, len: 0 }), GemErrorText::NetworkStatus { status: 503 });
    }

    #[test]
    fn test_a_wallet_connect_failure_reads_as_what_went_wrong_on_either_sdk() {
        assert_eq!(wallet_connect_error_text("The WalletConnect Pairing URI has expired.".to_string()), GemErrorText::ConnectionExpired);
        assert_eq!(wallet_connect_error_text("Pairing URI expired: 1700000000".to_string()), GemErrorText::ConnectionExpired);
        assert_eq!(wallet_connect_error_text("Session proposal expired".to_string()), GemErrorText::ConnectionExpired);
        assert_eq!(wallet_connect_error_text("There is no existing session matching the topic: abc.".to_string()), GemErrorText::ConnectionNotFound);
        assert_eq!(wallet_connect_error_text("Cannot find sequence for given topic: abc".to_string()), GemErrorText::ConnectionNotFound);
        assert_eq!(wallet_connect_error_text("Web socket is not connected to any URL or networking connection error".to_string()), GemErrorText::RelayUnavailable);
        assert_eq!(wallet_connect_error_text("Connection error: Please check your Internet connection".to_string()), GemErrorText::RelayUnavailable);
        assert_eq!(
            wallet_connect_error_text("Methods set is invalid.".to_string()),
            GemErrorText::Message { text: "Methods set is invalid.".to_string() },
            "anything else keeps the SDK's own words"
        );
    }
}
